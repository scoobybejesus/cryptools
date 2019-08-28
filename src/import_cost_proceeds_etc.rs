// Copyright (c) 2017-2019, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools-rs/blob/master/LEGAL.txt

use std::collections::{HashMap};
use std::error::Error;

use chrono::NaiveDate;
use decimal::d128;

use crate::transaction::{Transaction, TxType, ActionRecord, Polarity};
use crate::account::{Account, RawAccount};
use crate::decimal_utils::{round_d128_1e2};
use crate::core_functions::{ImportProcessParameters};

pub fn add_cost_basis_to_movements(
    settings: &ImportProcessParameters,
    ars: &HashMap<u32, ActionRecord>,
    raw_acct_map: &HashMap<u16, RawAccount>,
    acct_map: &HashMap<u16, Account>,
    txns_map: &HashMap<u32, Transaction>,
) -> Result<(), Box<Error>> {

    let length = txns_map.len();

    for txn_num in 1..=length {

        let txn_num = txn_num as u32;
        let txn = txns_map.get(&(txn_num)).unwrap();

        for ar_num in txn.action_record_idx_vec.iter() {

            let ar = ars.get(ar_num).unwrap();
            let acct = acct_map.get(&ar.account_key).unwrap();
            let raw_acct = raw_acct_map.get(&acct.raw_key).unwrap();
            let movements = ar.get_mvmts_in_ar(acct_map, txns_map);

            for mvmt in movements.iter() {

                let polarity = ar.direction();
                let tx_type = txn.transaction_type(ars, raw_acct_map, acct_map)?;
                let is_home_curr = raw_acct.is_home_currency(&settings.home_currency);
                let mvmt_copy = mvmt.clone();
                let borrowed_mvmt = mvmt_copy.clone();
                // println!("Txn: {} on {} of type: {:?}",txn.tx_number,txn.date, txn.transaction_type());

                if !raw_acct.is_margin {
                    match polarity {
                        Polarity::Outgoing => {
                            if is_home_curr {
                                let mvmts_amt = mvmt_copy.amount;
                                mvmt.cost_basis.set(mvmts_amt);
                            } else {
                                let mvmt_lot = mvmt_copy.get_lot(acct_map, ars);
                                let borrowed_mvmt_list = mvmt_lot.movements.borrow().clone();
                                let lots_first_mvmt = borrowed_mvmt_list.first().unwrap().clone();
                                let cb_of_lots_first_mvmt = lots_first_mvmt.cost_basis.get();
                                let ratio_of_amt_to_lots_first_mvmt = borrowed_mvmt.ratio_of_amt_to_lots_first_mvmt(acct_map, ars);
                                let unrounded_basis = -(cb_of_lots_first_mvmt * ratio_of_amt_to_lots_first_mvmt);
                                let rounded_basis = round_d128_1e2(&unrounded_basis);
                                mvmt.cost_basis.set(rounded_basis);
                                // if txn.tx_number == 5 {
                                // println!("Txn#: {}, ratio: {}, cb of lot's 1st mvmt: {}, cost basis: {}", txn.tx_number, ratio_of_amt_to_lots_first_mvmt, cb_of_lots_first_mvmt, cost_basis)
                                    // };
                            }
                            // println!("Outgoing a_r from txn: {} of type: {:?}, with {} {} with cost basis: {}",
                            //     txn.tx_number, txn.transaction_type(), borrowed_mvmt.amount, ar.account.ticker, cost_basis);
                            assert!(mvmt.cost_basis.get() <= d128!(0));
                            continue
                        }
                        Polarity::Incoming => {
                            if is_home_curr {
                                let mvmts_amt = mvmt_copy.amount;
                                mvmt.cost_basis.set(mvmts_amt);
                            } else {
                                match tx_type {
                                    TxType::Exchange => {
                                        let other_ar = ars.get(&txn.action_record_idx_vec[0]).unwrap();
                                        let other_acct = acct_map.get(&other_ar.account_key).unwrap();
                                        let raw_other_acct = raw_acct_map.get(&other_acct.raw_key).unwrap();
                                        assert_eq!(other_ar.direction(), Polarity::Outgoing);
                                        let other_ar_is_home_curr = raw_other_acct.is_home_currency(&settings.home_currency);

                                        if other_ar_is_home_curr {
                                            mvmt.cost_basis.set(-(other_ar.amount));
                                        } else {

                                            let ratio_of_amt_to_incoming_mvmts_in_a_r =
                                                borrowed_mvmt.ratio_of_amt_to_incoming_mvmts_in_a_r;
                                            let txn_proceeds = txn.proceeds.to_string().parse::<d128>().unwrap();
                                            let unrounded_basis = txn_proceeds * ratio_of_amt_to_incoming_mvmts_in_a_r;
                                            let rounded_basis = round_d128_1e2(&unrounded_basis);
                                            mvmt.cost_basis.set(rounded_basis);
                                        }
                                    }
                                    TxType::ToSelf => {
                                        let cb_outgoing_ar = retrieve_cost_basis_from_corresponding_outgoing_toself(
                                            txn_num, &ars, txns_map, acct_map);
                                        let ratio_of_amt_to_incoming_mvmts_in_a_r = borrowed_mvmt.ratio_of_amt_to_incoming_mvmts_in_a_r;
                                        let this_ratio = (ratio_of_amt_to_incoming_mvmts_in_a_r)
                                            .to_string()
                                            .parse::<d128>()
                                            .unwrap();
                                        let unrounded_basis = cb_outgoing_ar * this_ratio;
                                        let rounded_basis = round_d128_1e2(&unrounded_basis);
                                        mvmt.cost_basis.set(-rounded_basis);
                                    }
                                    TxType::Flow => {
                                        let txn_proceeds = txn.proceeds.to_string().parse::<d128>().unwrap();
                                        let mvmt_proceeds = round_d128_1e2(&(txn_proceeds *
                                            borrowed_mvmt.ratio_of_amt_to_incoming_mvmts_in_a_r));
                                        mvmt.cost_basis.set(mvmt_proceeds);
                                    }
                                }
                            }
                            // println!("Incoming a_r from txn: {} of type: {:?}, with {} {} with cost basis: {}",
                            //     txn.tx_number, txn.transaction_type(), borrowed_mvmt.amount, ar.account.ticker, cost_basis);
                            assert!(mvmt.cost_basis.get() >= d128!(0));
                            continue
                        }
                    }
                }
            }
        }
    }
    fn retrieve_cost_basis_from_corresponding_outgoing_toself(
        txn_num: u32,
        ars: &HashMap<u32, ActionRecord>,
        txns_map: &HashMap<u32, Transaction>,
        acct_map: &HashMap<u16, Account>,
    ) -> d128 {

        let txn = txns_map.get(&txn_num).unwrap();
        let other_ar_borrowed = &ars.get(&txn.action_record_idx_vec[0]).unwrap();

        assert_eq!(other_ar_borrowed.direction(), Polarity::Outgoing);

        let mut basis = d128!(0);
        let movements = other_ar_borrowed.get_mvmts_in_ar(acct_map, txns_map);

        for mvmt in movements.iter() {
            basis += mvmt.cost_basis.get();
        }

        basis
    };

    Ok(())
}

pub fn add_proceeds_to_movements(
    ars: &HashMap<u32, ActionRecord>,
    raw_acct_map: &HashMap<u16, RawAccount>,
    acct_map: &HashMap<u16, Account>,
    txns_map: &HashMap<u32, Transaction>,
) -> Result<(), Box<Error>> {

    let length = txns_map.len();

    for txn_num in 1..=length {

        let txn_num = txn_num as u32;
        let txn = txns_map.get(&(txn_num)).unwrap();

        for ar_num in txn.action_record_idx_vec.iter() {

            let ar = ars.get(ar_num).unwrap();
            let movements = ar.get_mvmts_in_ar(acct_map, txns_map);

            for mvmt in movements.iter() {

                let polarity = ar.direction();
                let tx_type = txn.transaction_type(ars, raw_acct_map, acct_map)?;
                let mvmt_copy = mvmt.clone();
                let borrowed_mvmt = mvmt_copy.clone();

                match tx_type {
                    TxType::Exchange => {
                        match polarity {
                            Polarity::Outgoing => {
                                let ratio = borrowed_mvmt.amount / ar.amount;
                                let proceeds_unrounded = txn.proceeds.to_string().parse::<d128>().unwrap() * ratio;
                                let proceeds_rounded = round_d128_1e2(&proceeds_unrounded);
                                mvmt.proceeds.set(proceeds_rounded);
                            }
                            Polarity::Incoming => {}
                        }
                    }
                    TxType::Flow => {
                        let ratio = borrowed_mvmt.amount / ar.amount;
                        let proceeds_unrounded = txn.proceeds.to_string().parse::<d128>().unwrap() * ratio;
                        let proceeds_rounded = round_d128_1e2(&proceeds_unrounded);
                        mvmt.proceeds.set(proceeds_rounded);
                    }
                    TxType::ToSelf => {}
                }
                // println!("Txn: {}, type: {:?} of {} {} w/ proceeds: {} & basis: {}",
                //     txn.tx_number, txn.transaction_type(), borrowed_mvmt.amount, ar.account.ticker, proceeds, borrowed_mvmt.cost_basis);
            }
        }
    }

    Ok(())
}

pub fn apply_like_kind_treatment(
    cutoff_date: NaiveDate,
    settings: &ImportProcessParameters,
    ars: &HashMap<u32, ActionRecord>,
    raw_acct_map: &HashMap<u16, RawAccount>,
    acct_map: &HashMap<u16, Account>,
    txns_map: &HashMap<u32, Transaction>,
) -> Result<(), Box<Error>> {

    let length = txns_map.len();
    for txn_num in 1..=length {

        let txn_num = txn_num as u32;
        let txn = txns_map.get(&(txn_num)).unwrap();

        update_current_txn_for_prior_likekind_treatment(txn_num, &settings, &ars, &raw_acct_map, &acct_map, &txns_map)?;

        if txn.date <= cutoff_date {
            perform_likekind_treatment_on_txn(txn_num, &settings, &ars, &raw_acct_map, &acct_map, &txns_map)?;
        }
    }

    Ok(())
}

fn update_current_txn_for_prior_likekind_treatment(
    txn_num: u32,
    settings: &ImportProcessParameters,
    ars: &HashMap<u32, ActionRecord>,
    raw_acct_map: &HashMap<u16, RawAccount>,
    acct_map: &HashMap<u16, Account>,
    txns_map: &HashMap<u32, Transaction>,
) -> Result<(), Box<Error>> {

    let mut sum_of_outgoing_cost_basis_in_ar = d128!(0);
    let txn = txns_map.get(&txn_num).unwrap();

    for ar_num in txn.action_record_idx_vec.iter() {

        let ar = ars.get(ar_num).unwrap();
        let acct = acct_map.get(&ar.account_key).unwrap();
        let raw_acct = raw_acct_map.get(&acct.raw_key).unwrap();
        let movements = ar.get_mvmts_in_ar(acct_map, txns_map);

        for mvmt in movements.iter() {

            let polarity = ar.direction();
            let tx_type = txn.transaction_type(ars, raw_acct_map, acct_map)?;
            let is_home_curr = raw_acct.is_home_currency(&settings.home_currency);

            let mvmt_copy = mvmt.clone();
            let borrowed_mvmt = mvmt_copy.clone();

            if !raw_acct.is_margin {
                match polarity {
                    Polarity::Outgoing => {
                        if !is_home_curr {
                            let borrowed_mvmt_lot = borrowed_mvmt.get_lot(acct_map, ars);
                            let borrowed_mvmt_list = borrowed_mvmt_lot.movements.borrow();
                            let cb_of_lots_first_mvmt = borrowed_mvmt_list.first().unwrap().cost_basis.get();
                            let ratio_of_amt_to_lots_first_mvmt = borrowed_mvmt.ratio_of_amt_to_lots_first_mvmt(acct_map, ars);
                            let unrounded_basis = -(cb_of_lots_first_mvmt * ratio_of_amt_to_lots_first_mvmt);
                            let rounded_basis = round_d128_1e2(&unrounded_basis);
                            mvmt.cost_basis.set(rounded_basis);
                        }
                        sum_of_outgoing_cost_basis_in_ar += mvmt.cost_basis.get()
                    }
                    Polarity::Incoming => {
                        match tx_type {
                            TxType::Exchange => {}
                            TxType::Flow => {}
                            TxType::ToSelf => {
                                if !is_home_curr {
                                    let ratio_of_amt_to_incoming_mvmts_in_a_r =
                                        borrowed_mvmt.ratio_of_amt_to_incoming_mvmts_in_a_r;
                                    let unrounded_basis = sum_of_outgoing_cost_basis_in_ar *
                                        ratio_of_amt_to_incoming_mvmts_in_a_r;
                                    let rounded_basis = round_d128_1e2(&unrounded_basis);
                                    mvmt.cost_basis.set(-rounded_basis);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn perform_likekind_treatment_on_txn(
    txn_num: u32,
    settings: &ImportProcessParameters,
    ars: &HashMap<u32, ActionRecord>,
    raw_acct_map: &HashMap<u16, RawAccount>,
    acct_map: &HashMap<u16, Account>,
    txns_map: &HashMap<u32, Transaction>,
) -> Result<(), Box<Error>> {

    let txn = txns_map.get(&txn_num).unwrap();
    let tx_type = txn.transaction_type(ars, raw_acct_map, acct_map)?;

    let og_ar = ars.get(&txn.action_record_idx_vec.first().unwrap()).unwrap();
    let ic_ar = ars.get(&txn.action_record_idx_vec.last().unwrap()).unwrap();
    let og_acct = acct_map.get(&og_ar.account_key).unwrap();
    let ic_acct = acct_map.get(&ic_ar.account_key).unwrap();
    let raw_og_acct = raw_acct_map.get(&og_acct.raw_key).unwrap();
    let raw_ic_acct = raw_acct_map.get(&ic_acct.raw_key).unwrap();

    fn both_are_non_home_curr(raw_og_acct: &RawAccount, raw_ic_acct: &RawAccount, settings: &ImportProcessParameters) -> bool {
        let og_is_home_curr = raw_og_acct.is_home_currency(&settings.home_currency);
        let ic_is_home_curr = raw_ic_acct.is_home_currency(&settings.home_currency);
        let both_are_non_home_curr = !ic_is_home_curr && !og_is_home_curr;
        both_are_non_home_curr
    }

    match tx_type {
        TxType::Exchange => {
            if both_are_non_home_curr(raw_og_acct, raw_ic_acct, settings) {
                let mut sum_of_outgoing_cost_basis_in_ar = d128!(0);
                for ar_num in txn.action_record_idx_vec.iter() {
                    let ar = ars.get(ar_num).unwrap();
                    let movements = ar.get_mvmts_in_ar(acct_map, txns_map);
                    for mvmt in movements.iter() {

                        let polarity = ar.direction();

                        let mvmt_copy = mvmt.clone();
                        let borrowed_mvmt = mvmt_copy.clone();

                        match polarity {
                            Polarity::Outgoing => {
                                let cb = borrowed_mvmt.cost_basis.get();
                                sum_of_outgoing_cost_basis_in_ar += cb;
                                mvmt.proceeds.set(-cb);
                            }
                            Polarity::Incoming => {
                                let ratio_of_amt_to_incoming_mvmts_in_a_r =
                                    borrowed_mvmt.ratio_of_amt_to_incoming_mvmts_in_a_r;
                                let unrounded_basis = sum_of_outgoing_cost_basis_in_ar *
                                    ratio_of_amt_to_incoming_mvmts_in_a_r;
                                let rounded_basis = round_d128_1e2(&unrounded_basis);
                                mvmt.cost_basis.set(-rounded_basis);
                            }
                        }
                    }
                }
            }
        }
        TxType::Flow => {
            if txn.action_record_idx_vec.len() == 2 {
                for ar_num in txn.action_record_idx_vec.iter() {

                    let ar = ars.get(ar_num).unwrap();
                    let movements = ar.get_mvmts_in_ar(acct_map, txns_map);

                    let polarity = ar.direction();

                    for mvmt in movements.iter() {

                        match polarity {
                            Polarity::Outgoing => {}
                            Polarity::Incoming => {
                                //  Reminder: May need extra logic here if margin exchange trades get cost_basis and proceeds
                                mvmt.cost_basis.set(d128!(0));
                                mvmt.proceeds.set(d128!(0));
                            }
                        }
                    }
                }
            }
        }
        TxType::ToSelf => {}
    }

    Ok(())
}
