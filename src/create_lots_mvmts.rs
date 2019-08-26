// Copyright (c) 2017-2019, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools-rs/blob/master/LEGAL.txt

use std::rc::{Rc};
use std::cell::{RefCell, Ref, Cell};
use std::collections::{HashMap};

use decimal::d128;
use chrono::NaiveDate;

use crate::transaction::{Transaction, ActionRecord, TxType, Polarity, TxHasMargin};
use crate::account::{Account, RawAccount, Lot, Movement};
use crate::core_functions::{InventoryCostingMethod, LikeKindSettings, ImportProcessParameters};
use crate::decimal_utils::{round_d128_1e8};

pub fn create_lots_and_movements(
    txns_map: HashMap<u32, Transaction>,
    settings: &ImportProcessParameters,
    likekind_settings: &Option<LikeKindSettings>,
    ar_map: &HashMap<u32, ActionRecord>,
    raw_acct_map: &HashMap<u16, RawAccount>,
    acct_map: &HashMap<u16, Account>,
    lot_map: &HashMap<(RawAccount, u32), Lot>,
) -> HashMap<u32,Transaction> {

    //  Set values to be referred to repeatedly, potentially, in Incoming Exchange transactions
    let multiple_incoming_mvmts_per_ar = match &likekind_settings {
        Some(likekind_settings) => {
            likekind_settings.like_kind_basis_date_preserved
        }
        None => {
            false
        }
    };

    let mut like_kind_cutoff_date: NaiveDate = NaiveDate::parse_from_str("1/1/1", "%m/%d/%y")
        .expect("NaiveDate string parsing failed. It shouldn't have. Try again.");
    // Above sets date never to be used.  Below modifies date in case it will be used.
    if likekind_settings.is_some() {
        let likekind_settings_clone = likekind_settings.clone().unwrap();
        like_kind_cutoff_date = likekind_settings_clone.like_kind_cutoff_date;
    }

    //  On with the creating of lots and movements.
    let length = txns_map.len();
    for num in 1..=length {

        let txn_num = num as u32;
        let txn = txns_map.get(&(txn_num)).expect("Couldn't get txn. Tx num invalid?");
        if txn.marginness(&ar_map, &raw_acct_map, &acct_map) == TxHasMargin::TwoARs {
            assert_eq!(txn.transaction_type(&ar_map, &raw_acct_map, &acct_map), TxType::Exchange);
            assert_eq!(txn.action_record_idx_vec.len(), 2);

            let the_raw_pair_keys = txn.get_base_and_quote_raw_acct_keys(&ar_map, &raw_acct_map, &acct_map);
            let base_acct = acct_map.get(&the_raw_pair_keys.0).expect("Couldn't get acct. Raw pair keys invalid?");
            let quote_acct = acct_map.get(&the_raw_pair_keys.1).expect("Couldn't get acct. Raw pair keys invalid?");

            let (base_ar_idx, quote_ar_idx) = get_base_and_quote_ar_idxs(
                the_raw_pair_keys,
                &txn,
                &ar_map,
                &raw_acct_map,
                &acct_map
            );

            let base_ar = ar_map.get(&base_ar_idx).unwrap();
            let quote_ar = ar_map.get(&quote_ar_idx).unwrap();

            let mut base_acct_lot_list = base_acct.list_of_lots.borrow_mut();
            let mut quote_acct_lot_list = quote_acct.list_of_lots.borrow_mut();

            let base_number_of_lots = base_acct_lot_list.len() as u32;
            let quote_number_of_lots = quote_acct_lot_list.len() as u32;
            assert_eq!(base_number_of_lots, quote_number_of_lots, "");

            let acct_balances_are_zero: bool;

            if !base_acct_lot_list.is_empty() && !quote_acct_lot_list.is_empty() {
                let base_balance_is_zero = base_acct_lot_list.last().unwrap().get_sum_of_amts_in_lot() == d128!(0);
                let quote_balance_is_zero = quote_acct_lot_list.last().unwrap().get_sum_of_amts_in_lot() == d128!(0);
                if base_balance_is_zero && quote_balance_is_zero {
                    acct_balances_are_zero = true
                } else {
                    acct_balances_are_zero = false
                }
            } else {
                assert_eq!(true, base_acct_lot_list.is_empty(),
                    "One margin account's list_of_lots is empty, but its pair's isn't.");
                assert_eq!(true, quote_acct_lot_list.is_empty(),
                    "One margin account's list_of_lots is empty, but its pair's isn't.");
                acct_balances_are_zero = true
            }

            let mut base_lot: Rc<Lot>;
            let mut quote_lot: Rc<Lot>;

            if acct_balances_are_zero {
                base_lot =
                Rc::new(
                    Lot {
                        date_as_string: txn.date_as_string.clone(),
                        date_of_first_mvmt_in_lot: txn.date,
                        date_for_basis_purposes: txn.date,
                        lot_number: base_number_of_lots + 1,
                        account_key: the_raw_pair_keys.0,
                        movements: RefCell::new([].to_vec()),
                    }
                )
                ;
                quote_lot =
                Rc::new(
                    Lot {
                        date_as_string: txn.date_as_string.clone(),
                        date_of_first_mvmt_in_lot: txn.date,
                        date_for_basis_purposes: txn.date,
                        lot_number: quote_number_of_lots + 1,
                        account_key: the_raw_pair_keys.1,
                        movements: RefCell::new([].to_vec()),
                    }
                )
                ;
            } else {
                base_lot = base_acct_lot_list.last().expect("Couldn't get lot. Base acct lot list empty?").clone();
                quote_lot = quote_acct_lot_list.last().expect("Couldn't get lot. Quote acct lot list empty?").clone();
            }

            let base_mvmt = Movement {
                amount: base_ar.amount,
                date_as_string: txn.date_as_string.clone(),
                date: txn.date,
                transaction_key: txn_num,
                action_record_key: base_ar_idx,
                cost_basis: Cell::new(d128!(0.0)),
                ratio_of_amt_to_incoming_mvmts_in_a_r: d128!(1.0),
                ratio_of_amt_to_outgoing_mvmts_in_a_r: Cell::new(d128!(1.0)),
                lot_num: base_lot.lot_number,
                proceeds: Cell::new(d128!(0.0)),
            };
            wrap_mvmt_and_push(base_mvmt, &base_ar, &base_lot, &settings, &raw_acct_map, &acct_map);

            let quote_mvmt = Movement {
                amount: quote_ar.amount,
                date_as_string: txn.date_as_string.clone(),
                date: txn.date,
                transaction_key: txn_num,
                action_record_key: quote_ar_idx,
                cost_basis: Cell::new(d128!(0.0)),
                ratio_of_amt_to_incoming_mvmts_in_a_r: d128!(1.0),
                ratio_of_amt_to_outgoing_mvmts_in_a_r: Cell::new(d128!(1.0)),
                lot_num: quote_lot.lot_number,
                proceeds: Cell::new(d128!(0.0)),
            };
            wrap_mvmt_and_push(quote_mvmt, &quote_ar, &quote_lot, &settings, &raw_acct_map, &acct_map);

            if acct_balances_are_zero {
                base_acct_lot_list.push(base_lot);
                quote_acct_lot_list.push(quote_lot);
            }
            continue
        } else {
            for ar_num in txn.action_record_idx_vec.iter() {
                let ar = ar_map.get(ar_num).unwrap();

                let acct = acct_map.get(&ar.account_key).unwrap();
                let raw_acct = raw_acct_map.get(&acct.raw_key).unwrap();
                let length_of_list_of_lots = acct.list_of_lots.borrow().len();

                if raw_acct.is_home_currency(&settings.home_currency) {
                    if length_of_list_of_lots == 0 {
                        let lot =
                        Rc::new(
                            Lot {
                                date_as_string: txn.date_as_string.clone(),
                                date_of_first_mvmt_in_lot: txn.date,
                                date_for_basis_purposes: txn.date,
                                lot_number: 1,
                                account_key: acct.raw_key,
                                movements: RefCell::new([].to_vec()),
                            }
                        )
                        ;
                        let whole_mvmt = Movement {
                            amount: ar.amount,
                            date_as_string: txn.date_as_string.clone(),
                            date: txn.date,
                            transaction_key: txn_num,
                            action_record_key: *ar_num,
                            cost_basis: Cell::new(d128!(0.0)),
                            ratio_of_amt_to_incoming_mvmts_in_a_r: d128!(1.0),
                            ratio_of_amt_to_outgoing_mvmts_in_a_r: Cell::new(d128!(1.0)),
                            lot_num: lot.lot_number,
                            proceeds: Cell::new(d128!(0.0)),
                        };
                        wrap_mvmt_and_push(whole_mvmt, &ar, &lot, &settings, &raw_acct_map, &acct_map);
                        acct.list_of_lots.borrow_mut().push(lot);
                        continue
                    }
                    else {
                        assert_eq!(1, length_of_list_of_lots); //  Only true for home currency
                        let lot = acct.list_of_lots.borrow_mut()[0 as usize].clone();
                        let whole_mvmt = Movement {
                            amount: ar.amount,
                            date_as_string: txn.date_as_string.clone(),
                            date: txn.date,
                            transaction_key: txn_num,
                            action_record_key: *ar_num,
                            cost_basis: Cell::new(d128!(0.0)),
                            ratio_of_amt_to_incoming_mvmts_in_a_r: d128!(1.0),
                            ratio_of_amt_to_outgoing_mvmts_in_a_r: Cell::new(d128!(1.0)),
                            lot_num: lot.lot_number,
                            proceeds: Cell::new(d128!(0.0)),
                        };
                        wrap_mvmt_and_push(whole_mvmt, &ar, &lot, &settings, &raw_acct_map, &acct_map);
                        continue
                    }
                }
                //  Note: a_r is not in home currency if here or below
                let polarity = ar.direction();
                let tx_type = txn.transaction_type(&ar_map, &raw_acct_map, &acct_map);

                match polarity {
                    Polarity::Outgoing => {
                        // println!("Txn: {}, outgoing {:?}-type of {} {}",
                            // txn.tx_number, txn.transaction_type(), ar.amount, acct.ticker);
                        //
                        if raw_acct.is_margin {
                            let this_acct = acct_map.get(&ar.account_key).unwrap();
                            let lot = this_acct.list_of_lots.borrow().last()
                                .expect("Couldn't get lot. Acct lot list empty?").clone();
                            let whole_mvmt = Movement {
                                amount: ar.amount,
                                date_as_string: txn.date_as_string.clone(),
                                date: txn.date,
                                transaction_key: txn_num,
                                action_record_key: *ar_num,
                                cost_basis: Cell::new(d128!(0.0)),
                                ratio_of_amt_to_incoming_mvmts_in_a_r: d128!(1.0),
                                ratio_of_amt_to_outgoing_mvmts_in_a_r: Cell::new(d128!(1.0)),
                                lot_num: lot.lot_number,
                                proceeds: Cell::new(d128!(0.0)),
                            };
                            wrap_mvmt_and_push(whole_mvmt, &ar, &lot, &settings, &raw_acct_map, &acct_map);
                            continue
                        } else {
                            let list_of_lots_to_use = acct.list_of_lots.clone();

                            //  the following returns vec to be iterated from beginning to end, which provides the index for the correct lot
                            let vec_of_ordered_index_values = match settings.costing_method {
                                InventoryCostingMethod::LIFObyLotCreationDate => {
                                    get_lifo_by_creation_date(&list_of_lots_to_use.borrow())}
                                InventoryCostingMethod::LIFObyLotBasisDate => {
                                    get_lifo_by_lot_basis_date(&list_of_lots_to_use.borrow())}
                                InventoryCostingMethod::FIFObyLotCreationDate => {
                                    get_fifo_by_creation_date(&list_of_lots_to_use.borrow())}
                                InventoryCostingMethod::FIFObyLotBasisDate => {
                                    get_fifo_by_lot_basis_date(&list_of_lots_to_use.borrow())}
                            };

                            fn get_lifo_by_creation_date(list_of_lots: &Ref<Vec<Rc<Lot>>>) -> Vec<usize> {
                                let mut vec_of_indexes = [].to_vec();
                                for (idx, lot) in list_of_lots.iter().enumerate() {
                                    vec_of_indexes.insert(0, idx)
                                }
                                let vec = vec_of_indexes;
                                vec
                            }

                            fn get_lifo_by_lot_basis_date(list_of_lots: &Ref<Vec<Rc<Lot>>>) -> Vec<usize> {
                                let mut reordered_vec = list_of_lots.clone().to_vec();
                                let length = reordered_vec.len();
                                for _ in 0..length {
                                    for j in 0..length-1 {
                                        if reordered_vec[j].date_for_basis_purposes > reordered_vec[j+1].date_for_basis_purposes {
                                            reordered_vec.swap(j, j+1)
                                        }
                                    }
                                }
                                let mut vec_of_indexes = [].to_vec();
                                for (idx, lot) in reordered_vec.iter().enumerate() {
                                    vec_of_indexes.insert(0, idx)
                                }
                                let vec = vec_of_indexes;
                                vec
                            }

                            fn get_fifo_by_creation_date(list_of_lots: &Ref<Vec<Rc<Lot>>>) -> Vec<usize> {
                                let mut vec_of_indexes = [].to_vec();
                                for (idx, lot) in list_of_lots.iter().enumerate() {
                                    vec_of_indexes.push(idx)
                                }
                                let vec = vec_of_indexes;
                                vec
                            }

                            fn get_fifo_by_lot_basis_date(list_of_lots: &Ref<Vec<Rc<Lot>>>) -> Vec<usize> {
                                let mut reordered_vec = list_of_lots.clone().to_vec();
                                let length = reordered_vec.len();
                                for _ in 0..length {
                                    for j in 0..length-1 {
                                        if reordered_vec[j].date_for_basis_purposes > reordered_vec[j+1].date_for_basis_purposes {
                                            reordered_vec.swap(j, j+1)
                                        }
                                    }
                                }
                                let mut vec_of_indexes = [].to_vec();
                                for (idx, lot) in reordered_vec.iter().enumerate() {
                                    vec_of_indexes.push(idx)
                                }
                                let vec = vec_of_indexes;
                                vec
                            }

                            let index_position: usize = 0;
                            let lot_index = vec_of_ordered_index_values[index_position];

                            let lot_to_use = list_of_lots_to_use.borrow()[lot_index].clone();
                            let whole_mvmt = Movement {
                                amount: ar.amount,
                                date_as_string: txn.date_as_string.clone(),
                                date: txn.date,
                                transaction_key: txn_num,
                                action_record_key: *ar_num,
                                cost_basis: Cell::new(d128!(0.0)),
                                ratio_of_amt_to_incoming_mvmts_in_a_r: d128!(1.0),
                                ratio_of_amt_to_outgoing_mvmts_in_a_r: Cell::new(d128!(1.0)),
                                lot_num: lot_to_use.lot_number,
                                proceeds: Cell::new(d128!(0.0)),
                            };

                            fit_into_lots(
                                acct.raw_key,
                                txn_num,
                                *ar_num,
                                whole_mvmt,
                                list_of_lots_to_use,
                                vec_of_ordered_index_values,
                                index_position,
                                &settings,
                                &ar_map,
                                &raw_acct_map,
                                &acct_map,
                            );
                            continue
                        }
                    }
                    Polarity::Incoming => {
                        // println!("Txn: {}, Incoming {:?}-type of {} {}",
                        //     txn.tx_number, txn.transaction_type(), ar.amount, acct.ticker);
                        match tx_type {
                            TxType::Flow => {
                                let mut lot: Rc<Lot>;
                                if raw_acct.is_margin {
                                    let this_acct = acct_map.get(&ar.account_key).unwrap();
                                    let lot_list = this_acct.list_of_lots.borrow_mut();
                                    lot = lot_list.last().unwrap().clone();

                                    let mvmt = Movement {
                                        amount: ar.amount,
                                        date_as_string: txn.date_as_string.clone(),
                                        date: txn.date,
                                        transaction_key: txn_num,
                                        action_record_key: *ar_num,
                                        cost_basis: Cell::new(d128!(0.0)),
                                        ratio_of_amt_to_incoming_mvmts_in_a_r: d128!(1.0),
                                        ratio_of_amt_to_outgoing_mvmts_in_a_r: Cell::new(d128!(1.0)),
                                        lot_num: lot.lot_number,
                                        proceeds: Cell::new(d128!(0.0)),
                                    };
                                    wrap_mvmt_and_push(mvmt, &ar, &lot, &settings, &raw_acct_map, &acct_map);
                                    continue
                                } else {
                                    let mvmt: Movement;
                                    if txn.action_record_idx_vec.len() == 1 {
                                        lot =
                                        Rc::new(
                                            Lot {
                                                date_as_string: txn.date_as_string.clone(),
                                                date_of_first_mvmt_in_lot: txn.date,
                                                date_for_basis_purposes: txn.date,

                                                lot_number: length_of_list_of_lots as u32 + 1,
                                                account_key: acct.raw_key,
                                                movements: RefCell::new([].to_vec()),
                                            }
                                        )
                                        ;
                                        mvmt = Movement {
                                            amount: ar.amount,
                                            date_as_string: txn.date_as_string.clone(),
                                            date: txn.date,
                                            transaction_key: txn_num,
                                            action_record_key: *ar_num,
                                            cost_basis: Cell::new(d128!(0.0)),
                                            ratio_of_amt_to_incoming_mvmts_in_a_r: d128!(1.0),
                                            ratio_of_amt_to_outgoing_mvmts_in_a_r: Cell::new(d128!(1.0)),
                                            lot_num: lot.lot_number,
                                            proceeds: Cell::new(d128!(0.0)),
                                        };
                                    } else {
                                        assert_eq!(txn.action_record_idx_vec.len(), 2);
                                        // create list of incoming/positive amounts(mvmts) in margin lot, add them
                                        let mut positive_mvmt_list: Vec<Rc<Movement>> = [].to_vec();
                                        let mut total_positive_amounts = d128!(0);

                                        let (base_acct_key, quote_acct_key) = get_base_and_quote_acct_for_dual_actionrecord_flow_tx(
                                            txn_num,
                                            &ar_map,
                                            &raw_acct_map,
                                            &acct_map,
                                            &txns_map,
                                        );

                                        let base_acct = acct_map.get(&base_acct_key).unwrap();
                                        let base_acct_lot = base_acct.list_of_lots.borrow().last().unwrap().clone();
                                        //  TODO: generalize this to work with margin shorts as well
                                        for mvmt in base_acct_lot.movements.borrow().iter() {
                                            if mvmt.amount > d128!(0) {
                                                // println!("In lot# {}, positive mvmt amount: {} {},",
                                                //     base_acct_lot.lot_number,
                                                //     mvmt.borrow().amount,
                                                //     base_acct_lot.account.raw.ticker);
                                                total_positive_amounts += mvmt.amount;
                                                positive_mvmt_list.push(mvmt.clone())
                                            }
                                        }
                                        let mut amounts_used = d128!(0);
                                        let mut percentages_used = d128!(0);

                                        for pos_mvmt in positive_mvmt_list.iter().take(positive_mvmt_list.len()-1) {
                                            let inner_lot =
                                            Rc::new(
                                                Lot {
                                                    date_as_string: txn.date_as_string.clone(),
                                                    date_of_first_mvmt_in_lot: txn.date,
                                                    date_for_basis_purposes: pos_mvmt.date,
                                                    lot_number: acct.list_of_lots.borrow().len() as u32 + 1,
                                                    account_key: acct.raw_key,
                                                    movements: RefCell::new([].to_vec()),
                                                }
                                            );
                                            let percentage_used = round_d128_1e8(&(pos_mvmt.amount/&total_positive_amounts));
                                            let amount_used = round_d128_1e8(&(ar.amount*percentage_used));
                                            let inner_mvmt = Movement {
                                                amount: amount_used,
                                                date_as_string: txn.date_as_string.clone(),
                                                date: txn.date,
                                                transaction_key: txn_num,
                                                action_record_key: *ar_num,
                                                cost_basis: Cell::new(d128!(0.0)),
                                                ratio_of_amt_to_incoming_mvmts_in_a_r: percentage_used,
                                                ratio_of_amt_to_outgoing_mvmts_in_a_r: Cell::new(d128!(1.0)),
                                                lot_num: inner_lot.lot_number,
                                                proceeds: Cell::new(d128!(0.0)),
                                            };
                                            wrap_mvmt_and_push(inner_mvmt, &ar, &inner_lot, &settings, &raw_acct_map, &acct_map);
                                            acct.list_of_lots.borrow_mut().push(inner_lot);
                                            // acct.push_lot(inner_lot);
                                            amounts_used += amount_used;
                                            percentages_used += percentage_used;
                                        }

                                        let final_mvmt = positive_mvmt_list.last().unwrap();
                                        lot =
                                        Rc::new(
                                            Lot {
                                                date_as_string: txn.date_as_string.clone(),
                                                date_of_first_mvmt_in_lot: txn.date,
                                                date_for_basis_purposes: final_mvmt.date,
                                                lot_number: acct.list_of_lots.borrow().len() as u32 + 1,
                                                account_key: acct.raw_key,
                                                movements: RefCell::new([].to_vec()),
                                            }
                                        )
                                        ;
                                        mvmt = Movement {
                                            amount: round_d128_1e8(&(ar.amount - amounts_used)),
                                            date_as_string: txn.date_as_string.clone(),
                                            date: txn.date,
                                            transaction_key: txn_num,
                                            action_record_key: *ar_num,
                                            cost_basis: Cell::new(d128!(0.0)),
                                            ratio_of_amt_to_incoming_mvmts_in_a_r: round_d128_1e8(&(d128!(1.0) - percentages_used)),
                                            ratio_of_amt_to_outgoing_mvmts_in_a_r: Cell::new(d128!(1.0)),
                                            lot_num: lot.lot_number,
                                            proceeds: Cell::new(d128!(0.0)),
                                        };
                                    }
                                    wrap_mvmt_and_push(mvmt, &ar, &lot, &settings, &raw_acct_map, &acct_map);
                                    acct.list_of_lots.borrow_mut().push(lot);
                                    continue
                                }
                            }
                            TxType::Exchange => {
                                let both_are_non_home_curr: bool;
                                let og_ar = ar_map.get(txn.action_record_idx_vec.first().unwrap()).unwrap();
                                let og_acct = acct_map.get(&og_ar.account_key).unwrap();
                                let og_raw_acct = raw_acct_map.get(&og_acct.raw_key).unwrap();
                                let ic_ar = ar;
                                let ic_raw_acct = raw_acct;
                                both_are_non_home_curr = !og_raw_acct.is_home_currency(&settings.home_currency)
                                                        && !ic_raw_acct.is_home_currency(&settings.home_currency);

                                if both_are_non_home_curr && multiple_incoming_mvmts_per_ar && (txn.date <= like_kind_cutoff_date) {
                                    process_multiple_incoming_lots_and_mvmts(
                                        txn_num,
                                        &og_ar,
                                        &ic_ar,
                                        &settings,
                                        *ar_num,
                                        &raw_acct_map,
                                        &acct_map,
                                        &txns_map,
                                        &ar_map,
                                    );
                                    continue
                                }

                                else {
                                    let lot =
                                    Rc::new(
                                        Lot {
                                            date_as_string: txn.date_as_string.clone(),
                                            date_of_first_mvmt_in_lot: txn.date,
                                            date_for_basis_purposes: txn.date,
                                            lot_number: length_of_list_of_lots as u32 + 1,
                                            account_key: acct.raw_key,
                                            movements: RefCell::new([].to_vec()),
                                        }
                                    )
                                    ;
                                    let whole_mvmt = Movement {
                                        amount: ar.amount,
                                        date_as_string: txn.date_as_string.clone(),
                                        date: txn.date,
                                        transaction_key: txn_num,
                                        action_record_key: *ar_num,
                                        cost_basis: Cell::new(d128!(0.0)),
                                        ratio_of_amt_to_incoming_mvmts_in_a_r: d128!(1.0),
                                        ratio_of_amt_to_outgoing_mvmts_in_a_r: Cell::new(d128!(1.0)),
                                        lot_num: lot.lot_number,
                                        proceeds: Cell::new(d128!(0.0)),
                                    };
                                    wrap_mvmt_and_push(whole_mvmt, &ar, &lot, &settings, &raw_acct_map, &acct_map);
                                    acct.list_of_lots.borrow_mut().push(lot);
                                    continue
                                }
                            }
                            TxType::ToSelf => {
                                if raw_acct.is_margin {
                                    { println!("\n Found margin actionrecord in toself txn # {} \n", txn.tx_number); use std::process::exit; exit(1) };
                                } else {
                                    process_multiple_incoming_lots_and_mvmts(
                                        txn_num,
                                        &ar_map.get(txn.action_record_idx_vec.first().unwrap()).unwrap(), // outgoing
                                        &ar, // incoming
                                        &settings,
                                        *ar_num,
                                        &raw_acct_map,
                                        &acct_map,
                                        &txns_map,
                                        &ar_map,
                                    );
                                }
                                continue
                            }
                        }
                    }
                }
            }    //  end for ar in txn.actionrecords
        }   //  end of tx does not have marginness of TwoARs
    }   //  end for txn in transactions
    txns_map
}

fn get_base_and_quote_acct_for_dual_actionrecord_flow_tx(
    txn_num: u32,
    ar_map: &HashMap<u32, ActionRecord>,
    raw_acct_map: &HashMap<u16, RawAccount>,
    acct_map: &HashMap<u16, Account>,
    txns_map: &HashMap<u32, Transaction>,
) -> (u16, u16) {

    let txn = txns_map.get(&txn_num).expect("Couldn't get txn. Tx num invalid?");
    let og_flow_ar = ar_map.get(txn.action_record_idx_vec.first().unwrap()).unwrap();
    // println!("Acct: {}, Amount: {}, Tx: {}, ar: {}",
    //     outgoing_flow_ar.account_key, outgoing_flow_ar.amount, outgoing_flow_ar.tx_key, outgoing_flow_ar.self_ar_key);
    let og_ar_mvmts_list = &og_flow_ar.get_mvmts_in_ar(acct_map, txns_map); // TODO: ... in margin profit, this just takes a list of one mvmt
    let og_ar_list_first_mvmt = &og_ar_mvmts_list.first().unwrap(); // TODO: then this takes the one mvmt
    let og_ar_list_first_mvmt_ar = ar_map.get(&og_ar_list_first_mvmt.action_record_key).unwrap();
    let og_ar_list_first_mvmt_ar_acct = acct_map.get(&og_ar_list_first_mvmt_ar.account_key).unwrap();
    let og_mvmt_lot = &og_ar_list_first_mvmt_ar_acct.list_of_lots.borrow()[(og_ar_list_first_mvmt.lot_num - 1) as usize];
    // let og_mvmt_lot_strong = &og_mvmt_lot;
    let og_mvmt_lot_mvmts = og_mvmt_lot.movements.borrow();
    let og_mvmt_lot_first_mvmt = &og_mvmt_lot_mvmts.first().unwrap();
    let txn_of_og_mvmt_lot_first_mvmt = txns_map.get(&og_mvmt_lot_first_mvmt.transaction_key).unwrap();
    let (base_key,quote_key) = txn_of_og_mvmt_lot_first_mvmt.get_base_and_quote_raw_acct_keys(
        ar_map,
        &raw_acct_map,
        &acct_map); // TODO: should this panic on margin loss?
	(base_key, quote_key)
}

fn get_base_and_quote_ar_idxs(
    pair_keys: (u16,u16),
    txn: &Transaction,
    ars: &HashMap<u32, ActionRecord>,
    raw_acct_map: &HashMap<u16, RawAccount>,
    acct_map: &HashMap<u16, Account>
    ) -> (u32, u32){

    let incoming_ar = ars.get(&txn.action_record_idx_vec[0]).unwrap();
    let incoming_acct = acct_map.get(&incoming_ar.account_key).unwrap();
    let raw_ic_acct = raw_acct_map.get(&incoming_acct.raw_key).unwrap();
    let compare = raw_acct_map.get(&pair_keys.0).unwrap();  //  key.0 is base, and key.1 is quote

    if raw_ic_acct == compare {
        (txn.action_record_idx_vec[0], txn.action_record_idx_vec[1])
    } else {
        (txn.action_record_idx_vec[1], txn.action_record_idx_vec[0])
    }
}

fn wrap_mvmt_and_push(
    this_mvmt: Movement,
    ar: &ActionRecord,
    lot: &Lot,
    settings: &ImportProcessParameters,
    raw_acct_map: &HashMap<u16, RawAccount>,
    acct_map: &HashMap<u16, Account>,
) {

    let acct = acct_map.get(&ar.account_key).unwrap();
    let raw_acct = raw_acct_map.get(&acct.raw_key).unwrap();

    if ar.direction() == Polarity::Outgoing && !raw_acct.is_home_currency(&settings.home_currency) {
        let ratio = this_mvmt.amount / ar.amount;
        this_mvmt.ratio_of_amt_to_outgoing_mvmts_in_a_r.set(round_d128_1e8(&ratio));
    }

    let amt = this_mvmt.amount;
    let amt2 = round_d128_1e8(&amt);
    assert_eq!(amt, amt2);
    // println!("Unrounded: {}; Rounded: {}; on {}", amt, amt2, mvmt_ref.borrow().date);

    let mvmt = Rc::from(this_mvmt);
    lot.movements.borrow_mut().push(mvmt.clone());
    ar.movements.borrow_mut().push(mvmt);
}

fn fit_into_lots(
    acct_key: u16,
    txn_num: u32,
    spawning_ar_key: u32,
    mvmt_to_fit: Movement,
    list_of_lots_to_use: RefCell<Vec<Rc<Lot>>>,
    vec_of_ordered_index_values: Vec<usize>,
    index_position: usize,
    settings: &ImportProcessParameters,
    ar_map: &HashMap<u32, ActionRecord>,
    raw_acct_map: &HashMap<u16, RawAccount>,
    acct_map: &HashMap<u16, Account>,
    ) {

    let ar = ar_map.get(&spawning_ar_key).unwrap();
    let acct = acct_map.get(&ar.account_key).unwrap();
    let raw_acct = raw_acct_map.get(&acct.raw_key).unwrap();
    assert_eq!(raw_acct.is_home_currency(&settings.home_currency), false);

    let spawning_ar = ar_map.get(&spawning_ar_key).unwrap();
    let mut current_index_position = index_position;

    let lot = mvmt_to_fit.get_lot(acct_map, ar_map);

    let mut mut_sum_of_mvmts_in_lot: d128 = d128!(0.0);
    for movement in lot.movements.borrow().iter() {
        mut_sum_of_mvmts_in_lot += movement.amount;
    }
    let sum_of_mvmts_in_lot = mut_sum_of_mvmts_in_lot;
    assert!(sum_of_mvmts_in_lot >= d128!(0.0));

    if sum_of_mvmts_in_lot == d128!(0.0) { //  If the lot is "full", go to the next
        current_index_position += 1;
        assert!(current_index_position < vec_of_ordered_index_values.len());
        let lot_index = vec_of_ordered_index_values[current_index_position];
        let newly_chosen_lot = list_of_lots_to_use.borrow()[lot_index].clone();
        let possible_mvmt_to_fit = Movement {
            amount: mvmt_to_fit.amount,
            date_as_string: mvmt_to_fit.date_as_string.clone(),
            date: mvmt_to_fit.date,
            transaction_key: mvmt_to_fit.transaction_key,
            action_record_key: mvmt_to_fit.action_record_key,
            cost_basis: mvmt_to_fit.cost_basis,
            ratio_of_amt_to_incoming_mvmts_in_a_r: d128!(1.0), //  Outgoing mvmt, so it's irrelevant
            ratio_of_amt_to_outgoing_mvmts_in_a_r: Cell::new(d128!(1.0)),
            lot_num: newly_chosen_lot.lot_number,
            proceeds: Cell::new(d128!(0.0)),
        };
        fit_into_lots(
            acct.raw_key,
            txn_num,
            spawning_ar_key,
            possible_mvmt_to_fit,
            list_of_lots_to_use,
            vec_of_ordered_index_values,
            current_index_position,
            &settings,
            &ar_map,
            &raw_acct_map,
            &acct_map
        );
        return;
    }
    assert!(sum_of_mvmts_in_lot > d128!(0.0));
    let remainder_amt = mvmt_to_fit.amount;
    // println!("Sum of mvmts in lot: {}; Remainder amount: {}; Net: {}",
    //     sum_of_mvmts_in_lot, remainder_amt, sum_of_mvmts_in_lot + remainder_amt);

    let does_remainder_fit: bool = (sum_of_mvmts_in_lot + remainder_amt) >= d128!(0.0);

    if does_remainder_fit {
        let remainder_that_fits = Movement {
            amount: mvmt_to_fit.amount,
            date_as_string: mvmt_to_fit.date_as_string.clone(),
            date: mvmt_to_fit.date,
            transaction_key: mvmt_to_fit.transaction_key,
            action_record_key: mvmt_to_fit.action_record_key,
            cost_basis: mvmt_to_fit.cost_basis,
            ratio_of_amt_to_incoming_mvmts_in_a_r: d128!(1.0), //  Outgoing mvmt, so it's irrelevant
            ratio_of_amt_to_outgoing_mvmts_in_a_r: Cell::new(d128!(1.0)),
            lot_num: lot.lot_number,
            proceeds: Cell::new(d128!(0.0)),
        };
        wrap_mvmt_and_push(remainder_that_fits, &spawning_ar, &lot, &settings, &raw_acct_map, &acct_map);
        return  //  And we're done
    }
    //  Note: at this point, we know the movement doesn't fit in a single lot & sum_of_mvmts_in_lot > 0
    let mvmt = RefCell::new(mvmt_to_fit);
    let mvmt_rc = Rc::from(mvmt);

    let mvmt_that_fits_in_lot = Movement {
        amount: (-sum_of_mvmts_in_lot).reduce(),
        date_as_string: mvmt_rc.borrow().date_as_string.clone(),
        date: mvmt_rc.borrow().date,
        transaction_key: txn_num,
        action_record_key: spawning_ar_key,
        cost_basis: Cell::new(d128!(0.0)),
        ratio_of_amt_to_incoming_mvmts_in_a_r: d128!(1.0), //  Outgoing mvmt, so it's irrelevant
        ratio_of_amt_to_outgoing_mvmts_in_a_r: Cell::new(d128!(1.0)),
        lot_num: lot.lot_number,
        proceeds: Cell::new(d128!(0.0)),
    };
    wrap_mvmt_and_push(mvmt_that_fits_in_lot, &spawning_ar, &lot, &settings, &raw_acct_map, &acct_map);
    let remainder_amt_to_recurse = remainder_amt + sum_of_mvmts_in_lot;
    // println!("Remainder amount to recurse: {}", remainder_amt_to_recurse);
    current_index_position += 1;
    let lot_index = vec_of_ordered_index_values[current_index_position];
    let newly_chosen_lot = list_of_lots_to_use.borrow()[lot_index].clone();

    let remainder_mvmt_to_recurse = Movement {
        amount: remainder_amt_to_recurse.reduce(),
        date_as_string: mvmt_rc.borrow().date_as_string.clone(),
        date: mvmt_rc.borrow().date,
        transaction_key: txn_num,
        action_record_key: spawning_ar_key,
        cost_basis: Cell::new(d128!(0.0)), //  This acts as a dummy value.
        ratio_of_amt_to_incoming_mvmts_in_a_r: d128!(1.0), //  Outgoing mvmt, so it's irrelevant
        ratio_of_amt_to_outgoing_mvmts_in_a_r: Cell::new(d128!(1.0)), //  This acts as a dummy value.
        lot_num: newly_chosen_lot.lot_number,
        proceeds: Cell::new(d128!(0.0)),
    };
    assert!(current_index_position < vec_of_ordered_index_values.len());
    fit_into_lots(
        acct.raw_key,
        txn_num,
        spawning_ar_key,
        remainder_mvmt_to_recurse,
        list_of_lots_to_use,
        vec_of_ordered_index_values,
        current_index_position,
        &settings,
        &ar_map,
        &raw_acct_map,
        &acct_map
    );
}

fn process_multiple_incoming_lots_and_mvmts(
    txn_num: u32,
    outgoing_ar: &ActionRecord,
    incoming_ar: &ActionRecord,
    settings: &ImportProcessParameters,
    incoming_ar_key: u32,
    raw_acct_map: &HashMap<u16, RawAccount>,
    acct_map: &HashMap<u16, Account>,
    txns_map: &HashMap<u32, Transaction>,
    ar_map: &HashMap<u32, ActionRecord>,
) {

    let round_to_places = d128::from(1).scaleb(d128::from(-8));
    let txn = txns_map.get(&txn_num).expect("Couldn't get txn. Tx num invalid?");

    let acct_of_incoming_ar = acct_map.get(&incoming_ar.account_key).unwrap();

    let mut all_but_last_incoming_mvmt_amt = d128!(0.0);
    let mut all_but_last_incoming_mvmt_ratio = d128!(0.0);
    // println!("Txn date: {}. Outgoing mvmts: {}, Outgoing amount: {}", txn.date, outgoing_ar.movements.borrow().len(), outgoing_ar.amount);
    let list_of_mvmts_of_outgoing_ar = outgoing_ar.get_mvmts_in_ar(acct_map, txns_map);
    let final_mvmt = list_of_mvmts_of_outgoing_ar.last().unwrap();
    //  First iteration, for all but final movement
    for outgoing_mvmt in list_of_mvmts_of_outgoing_ar
                            .iter()
                            .take(outgoing_ar.get_mvmts_in_ar(acct_map, txns_map).len() - 1) {
        let ratio_of_outgoing_mvmt_to_total_ar = outgoing_mvmt.amount / outgoing_ar.amount; //  Negative divided by negative is positive
        // println!("Ratio of outgoing amt to total actionrecord amt: {:.8}", ratio_of_outgoing_to_total_ar);
        let tentative_incoming_amt = ratio_of_outgoing_mvmt_to_total_ar * incoming_ar.amount;
        // println!("Unrounded incoming amt: {}", tentative_incoming_amt);
        let corresponding_incoming_amt = tentative_incoming_amt.quantize(round_to_places);
        // println!("Rounded incoming amt: {}", corresponding_incoming_amt);
        if corresponding_incoming_amt == d128!(0) { continue }  //  Due to rounding, this could be zero.
        assert!(corresponding_incoming_amt > d128!(0.0));
        let this_acct = acct_of_incoming_ar;
        let length_of_list_of_lots: usize = this_acct.list_of_lots.borrow().len();
        let inherited_date = outgoing_mvmt.get_lot(acct_map, ar_map).date_of_first_mvmt_in_lot;
        let lot =
        Rc::new(
            Lot {
                date_as_string: txn.date_as_string.clone(),
                date_of_first_mvmt_in_lot: txn.date,
                date_for_basis_purposes: inherited_date,
                lot_number: length_of_list_of_lots as u32 + 1,
                account_key: this_acct.raw_key,
                movements: RefCell::new([].to_vec()),
            }
        )
        ;
        let incoming_mvmt = Movement {
            amount: corresponding_incoming_amt.reduce(),
            date_as_string: txn.date_as_string.clone(),
            date: txn.date,
            transaction_key: txn_num,
            action_record_key: incoming_ar_key,
            cost_basis: Cell::new(d128!(0.0)),
            ratio_of_amt_to_incoming_mvmts_in_a_r: round_d128_1e8(&ratio_of_outgoing_mvmt_to_total_ar),
            ratio_of_amt_to_outgoing_mvmts_in_a_r: Cell::new(d128!(1.0)),
            lot_num: lot.lot_number,
            proceeds: Cell::new(d128!(0.0)),
        };
        // println!("From first set of incoming movements, amount: {} {} to account: {}",
        //     incoming_mvmt.amount, acct_incoming_ar.ticker, acct_incoming_ar.account_num);
        all_but_last_incoming_mvmt_ratio += round_d128_1e8(&ratio_of_outgoing_mvmt_to_total_ar);
        all_but_last_incoming_mvmt_amt += incoming_mvmt.amount;
        wrap_mvmt_and_push(incoming_mvmt, &incoming_ar, &lot, &settings, &raw_acct_map, &acct_map);
        this_acct.list_of_lots.borrow_mut().push(lot);
    }
    //  Second iteration, for final movement
    let corresponding_incoming_amt = incoming_ar.amount - all_but_last_incoming_mvmt_amt;
    assert!(corresponding_incoming_amt > d128!(0.0));
    let this_acct = acct_of_incoming_ar;
    let length_of_list_of_lots = this_acct.list_of_lots.borrow().len();
    let inherited_date = final_mvmt.get_lot(acct_map, ar_map).date_of_first_mvmt_in_lot;
    let lot =
    Rc::new(
        Lot {
            date_as_string: txn.date_as_string.clone(),
            date_of_first_mvmt_in_lot: txn.date,
            date_for_basis_purposes: inherited_date,
            lot_number: length_of_list_of_lots as u32 + 1,
            account_key: this_acct.raw_key,
            movements: RefCell::new([].to_vec()),
        }
    )
    ;
    let incoming_mvmt = Movement {
        amount: corresponding_incoming_amt.reduce(),
        date_as_string: txn.date_as_string.clone(),
        date: txn.date,
        transaction_key: txn_num,
        action_record_key: incoming_ar_key,
        cost_basis: Cell::new(d128!(0.0)),
        ratio_of_amt_to_incoming_mvmts_in_a_r: d128!(1.0) - all_but_last_incoming_mvmt_ratio,
        ratio_of_amt_to_outgoing_mvmts_in_a_r: Cell::new(d128!(1.0)),
        lot_num: lot.lot_number,
        proceeds: Cell::new(d128!(0.0)),
    };
    // println!("Final incoming mvmt for this actionrecord, amount: {} {} to account: {}",
    //     incoming_mvmt.amount, acct_incoming_ar.ticker, acct_incoming_ar.account_num);
    wrap_mvmt_and_push(incoming_mvmt, &incoming_ar, &lot, &settings, &raw_acct_map, &acct_map);
    this_acct.list_of_lots.borrow_mut().push(lot);
}
