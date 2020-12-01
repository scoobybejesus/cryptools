// Copyright (c) 2017-2020, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools/blob/master/LEGAL.txt

use std::collections::{HashMap};
use std::error::Error;

use decimal::d128;

use crate::transaction::{Transaction, TxType, ActionRecord, Polarity};
use crate::account::{Account, RawAccount};
use crate::decimal_utils::{round_d128_1e2};
use crate::core_functions::{ImportProcessParameters};

pub(crate) fn add_cost_basis_to_movements(
    settings: &ImportProcessParameters,
    raw_acct_map: &HashMap<u16, RawAccount>,
    acct_map: &HashMap<u16, Account>,
    ars: &HashMap<u32, ActionRecord>,
    txns_map: &HashMap<u32, Transaction>,
) -> Result<(), Box<dyn Error>> {

    let length = txns_map.len();

    for txn_num in 1..=length {

        let txn_num = txn_num as u32;
        let txn = txns_map.get(&(txn_num)).unwrap();

        for ar_num in txn.action_record_idx_vec.iter() {

            let ar = ars.get(ar_num).unwrap();
            let acct = acct_map.get(&ar.account_key).unwrap();
            let raw_acct = raw_acct_map.get(&acct.raw_key).unwrap();
            let movements = ar.get_mvmts_in_ar_in_lot_date_order(acct_map, txns_map);

            for (idx, mvmt) in movements.iter().enumerate() {

                let polarity = ar.direction();
                let tx_type = txn.transaction_type(ars, raw_acct_map, acct_map)?;
                let is_home_curr = raw_acct.is_home_currency(&settings.home_currency);
                let mvmt_copy = mvmt.clone();
                let borrowed_mvmt = mvmt_copy.clone();
                // println!("Txn: {} on {} of type: {:?}",
                //     txn.tx_number,txn.date, txn.transaction_type(ars, raw_acct_map, acct_map));

                if !raw_acct.is_margin {

                    match polarity {

                        Polarity::Outgoing => {

                            if is_home_curr {

                                let mvmts_amt = mvmt_copy.amount;

                                mvmt.cost_basis.set(mvmts_amt);
                                mvmt.cost_basis_lk.set(mvmts_amt);

                            } else {

                                let cb_of_lots_first_mvmt = mvmt_copy.get_cost_basis_of_lots_first_mvmt(acct_map, ars);
                                let ratio_of_amt_to_lots_first_mvmt = borrowed_mvmt.ratio_of_amt_to_lots_first_mvmt(acct_map, ars);
                                let unrounded_basis = -(cb_of_lots_first_mvmt * ratio_of_amt_to_lots_first_mvmt);
                                let rounded_basis = round_d128_1e2(&unrounded_basis);

                                mvmt.cost_basis.set(rounded_basis);
                                mvmt.cost_basis_lk.set(rounded_basis);
                            }
                            assert!(mvmt.cost_basis.get() <= d128!(0));
                            // assert!(mvmt.cost_basis_lk.get() <= d128!(0));   //  Same as above assert.
                            continue
                        }

                        Polarity::Incoming => {

                            if is_home_curr {

                                let mvmts_amt = mvmt_copy.amount;

                                mvmt.cost_basis.set(mvmts_amt);
                                mvmt.cost_basis_lk.set(mvmts_amt);

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
                                            mvmt.cost_basis_lk.set(-(other_ar.amount));

                                        } else {

                                            let ratio_of_amt_to_incoming_mvmts_in_a_r =
                                                borrowed_mvmt.ratio_of_amt_to_incoming_mvmts_in_a_r;
                                            let txn_proceeds = txn.proceeds
                                                .to_string()
                                                .parse::<d128>()
                                                .unwrap();
                                            let unrounded_basis = txn_proceeds * ratio_of_amt_to_incoming_mvmts_in_a_r;
                                            let rounded_basis = round_d128_1e2(&unrounded_basis);

                                            mvmt.cost_basis.set(rounded_basis);
                                            mvmt.cost_basis_lk.set(rounded_basis);
                                        }
                                    }

                                    TxType::ToSelf => {

                                        let cb_vec_outgoing_ar = retrieve_cb_vec_from_corresponding_outgoing_toself(
                                            txn_num,
                                            &ars,
                                            txns_map,
                                            acct_map
                                        );

                                        assert!(idx <= cb_vec_outgoing_ar.len(),
                                            "ToSelf txn had different # of in- and out- mvmts (more outs than ins).");

                                        let unrounded_basis = cb_vec_outgoing_ar[idx];
                                        let rounded_basis = round_d128_1e2(&unrounded_basis);

                                        mvmt.cost_basis.set(-rounded_basis);
                                        mvmt.cost_basis_lk.set(-rounded_basis);
                                    }

                                    TxType::Flow => {

                                        let txn_proceeds = txn.proceeds.to_string().parse::<d128>().unwrap();
                                        let mvmt_proceeds = round_d128_1e2(
                                            &(txn_proceeds *
                                            borrowed_mvmt.ratio_of_amt_to_incoming_mvmts_in_a_r)
                                        );  //  Ratio should always be 1.0, but we do the calc anyway, for future-proofing.

                                        mvmt.cost_basis.set(mvmt_proceeds);
                                        mvmt.cost_basis_lk.set(mvmt_proceeds);
                                    }
                                }
                            }
                            assert!(mvmt.cost_basis.get() >= d128!(0));
                            // assert!(mvmt.cost_basis_lk.get() >= d128!(0));   //  Same as above assert.
                            continue
                        }
                    }
                } else {
                    // Do nothing. Future changes can add a code path where margin txns "settle"
                    // as they happen, though, if desired. Just need to write the code.
                }
            }
        }
    }

    fn retrieve_cb_vec_from_corresponding_outgoing_toself(
        txn_num: u32,
        ars: &HashMap<u32, ActionRecord>,
        txns_map: &HashMap<u32, Transaction>,
        acct_map: &HashMap<u16, Account>,
    ) -> Vec<d128> {

        let txn = txns_map.get(&txn_num).unwrap();
        let other_ar_borrowed = &ars.get(&txn.action_record_idx_vec[0]).unwrap();

        assert_eq!(other_ar_borrowed.direction(), Polarity::Outgoing);

        let movements = other_ar_borrowed.get_mvmts_in_ar_in_lot_date_order(acct_map, txns_map);
        let mut vec = Vec::new();

        for mvmt in movements.iter() {
            vec.push(mvmt.cost_basis.get());
        }

        vec
    };

    Ok(())
}

pub(crate) fn add_proceeds_to_movements(
    raw_acct_map: &HashMap<u16, RawAccount>,
    acct_map: &HashMap<u16, Account>,
    ars: &HashMap<u32, ActionRecord>,
    txns_map: &HashMap<u32, Transaction>,
) -> Result<(), Box<dyn Error>> {

    let length = txns_map.len();

    for txn_num in 1..=length {

        let txn_num = txn_num as u32;
        let txn = txns_map.get(&(txn_num)).unwrap();

        for ar_num in txn.action_record_idx_vec.iter() {

            let ar = ars.get(ar_num).unwrap();
            let acct = acct_map.get(&ar.account_key).unwrap();
            let raw_acct = raw_acct_map.get(&acct.raw_key).unwrap();
            let movements = ar.get_mvmts_in_ar_in_lot_date_order(acct_map, txns_map);

            if !raw_acct.is_margin {

                for mvmt in movements.iter() {

                    let polarity = ar.direction();
                    let tx_type = txn.transaction_type(ars, raw_acct_map, acct_map)?;
                    let mvmt_copy = mvmt.clone();
                    let borrowed_mvmt = mvmt_copy.clone();

                    match tx_type {

                        TxType::Exchange | TxType::Flow => {

                            match polarity {

                                Polarity::Outgoing => {

                                    if (tx_type == TxType::Flow) && (txn.action_record_idx_vec.len() == 2) {

                                        // Keep at 0.00 proceeds for margin loss
                                        continue
                                    }

                                    let ratio = borrowed_mvmt.amount / ar.amount;
                                    let proceeds_unrounded = txn.proceeds.to_string().parse::<d128>().unwrap() * ratio;
                                    let proceeds_rounded = round_d128_1e2(&proceeds_unrounded);

                                    mvmt.proceeds.set(proceeds_rounded);
                                    mvmt.proceeds_lk.set(proceeds_rounded);

                                }

                                Polarity::Incoming => {
                                    // For a time, this was blank. As part of the commit(s) to add cost_basis_lk
                                    // and proceeds_lk, let's change this to reflect that incoming proceeds are now
                                    // negative, which net against the positive cost_basis to result in a gain of $0.
                                    // Additionally, we apply the same treatment to Flow txns.
                                    mvmt.proceeds.set(-mvmt.cost_basis.get());
                                    mvmt.proceeds_lk.set(-mvmt.cost_basis_lk.get());
                                }
                            }
                        }

                        TxType::ToSelf => {
                            // Originally did nothing. Now explicity creating a condition where a report containing
                            // ToSelf txns would reflect a $0 gain/loss.
                            mvmt.proceeds.set(-mvmt.cost_basis.get());
                            mvmt.proceeds_lk.set(-mvmt.cost_basis_lk.get());
                        }
                    }
                }
            } else {
                // Do nothing. Future changes can add a code path where margin txns "settle"
                // as they happen, though, if desired. Just need to write the code.
            }
        }
    }

    Ok(())
}

pub(crate) fn apply_like_kind_treatment(
    settings: &ImportProcessParameters,
    raw_acct_map: &HashMap<u16, RawAccount>,
    acct_map: &HashMap<u16, Account>,
    ars: &HashMap<u32, ActionRecord>,
    txns_map: &HashMap<u32, Transaction>,
) -> Result<(), Box<dyn Error>> {

    let length = txns_map.len();
    let cutoff_date = settings.lk_cutoff_date;
    for txn_num in 1..=length {

        let txn_num = txn_num as u32;
        let txn = txns_map.get(&(txn_num)).unwrap();

        update_current_txn_for_prior_likekind_treatment(txn_num, &settings, &raw_acct_map, &acct_map, &ars, &txns_map)?;

        if txn.date <= cutoff_date {
            perform_likekind_treatment_on_txn(txn_num, &settings, &raw_acct_map, &acct_map, &ars, &txns_map)?;
        }
    }

    Ok(())
}

fn update_current_txn_for_prior_likekind_treatment(
    txn_num: u32,
    settings: &ImportProcessParameters,
    raw_acct_map: &HashMap<u16, RawAccount>,
    acct_map: &HashMap<u16, Account>,
    ars: &HashMap<u32, ActionRecord>,
    txns_map: &HashMap<u32, Transaction>,
) -> Result<(), Box<dyn Error>> {

    let mut sum_of_outgoing_lk_cost_basis_in_ar = d128!(0);
    let txn = txns_map.get(&txn_num).unwrap();

    for ar_num in txn.action_record_idx_vec.iter() {

        let ar = ars.get(ar_num).unwrap();
        let acct = acct_map.get(&ar.account_key).unwrap();
        let raw_acct = raw_acct_map.get(&acct.raw_key).unwrap();
        let movements = ar.get_mvmts_in_ar_in_lot_date_order(acct_map, txns_map);

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

                            let lk_cb_of_lots_first_mvmt = borrowed_mvmt.get_lk_cost_basis_of_lots_first_mvmt(acct_map, ars);
                            let ratio_of_amt_to_lots_first_mvmt = borrowed_mvmt.ratio_of_amt_to_lots_first_mvmt(acct_map, ars);
                            let unrounded_lk_basis = -(lk_cb_of_lots_first_mvmt * ratio_of_amt_to_lots_first_mvmt);
                            let rounded_lk_basis = round_d128_1e2(&unrounded_lk_basis);

                            mvmt.cost_basis_lk.set(rounded_lk_basis);

                            if tx_type == TxType::ToSelf {
                                mvmt.proceeds_lk.set(-rounded_lk_basis)
                            }
                        }
                        sum_of_outgoing_lk_cost_basis_in_ar += mvmt.cost_basis_lk.get()
                    }

                    Polarity::Incoming => {

                        match tx_type {

                            TxType::Exchange => {
                                // Do nothing.
                                // If txn.date is after the LK treatment date, the incoming mvmt goes untreated.
                                // If txn.date is before the LK date, incoming mvmt gets treatment in apply_lk_treatment next.
                            }
                            TxType::Flow => {
                                if txn.action_record_idx_vec.len() == 2 {
                                    mvmt.cost_basis_lk.set(d128!(0));
                                    mvmt.proceeds_lk.set(d128!(0));
                                }
                                // Do nothing for non-margin txns.
                            }
                            TxType::ToSelf => {

                                if !is_home_curr {

                                    let ratio_of_amt_to_incoming_mvmts_in_a_r =
                                        borrowed_mvmt.ratio_of_amt_to_incoming_mvmts_in_a_r;
                                    let unrounded_lk_basis = sum_of_outgoing_lk_cost_basis_in_ar *
                                        ratio_of_amt_to_incoming_mvmts_in_a_r;
                                    let rounded_lk_basis = round_d128_1e2(&unrounded_lk_basis);

                                    mvmt.cost_basis_lk.set(-rounded_lk_basis);
                                    mvmt.proceeds_lk.set(rounded_lk_basis);
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
    raw_acct_map: &HashMap<u16, RawAccount>,
    acct_map: &HashMap<u16, Account>,
    ars: &HashMap<u32, ActionRecord>,
    txns_map: &HashMap<u32, Transaction>,
) -> Result<(), Box<dyn Error>> {

    let txn = txns_map.get(&txn_num).unwrap();
    let tx_type = txn.transaction_type(ars, raw_acct_map, acct_map)?;
    let home_currency = &settings.home_currency;

    match tx_type {

        TxType::Exchange => {

            if txn.both_exch_ars_are_non_home_curr(ars, raw_acct_map, acct_map, home_currency)? {

                let mut sum_of_outgoing_lk_cost_basis_in_ar = d128!(0);

                for ar_num in txn.action_record_idx_vec.iter() {

                    let ar = ars.get(ar_num).unwrap();
                    let movements = ar.get_mvmts_in_ar_in_lot_date_order(acct_map, txns_map);

                    for mvmt in movements.iter() {

                        let polarity = ar.direction();

                        let mvmt_copy = mvmt.clone();
                        let borrowed_mvmt = mvmt_copy.clone();

                        match polarity {

                            Polarity::Outgoing => {

                                let cb = borrowed_mvmt.cost_basis_lk.get();
                                sum_of_outgoing_lk_cost_basis_in_ar += cb;

                                mvmt.proceeds_lk.set(-cb);
                            }

                            Polarity::Incoming => {

                                let ratio_of_amt_to_incoming_mvmts_in_a_r =
                                    borrowed_mvmt.ratio_of_amt_to_incoming_mvmts_in_a_r;
                                let unrounded_basis = sum_of_outgoing_lk_cost_basis_in_ar *
                                    ratio_of_amt_to_incoming_mvmts_in_a_r;
                                let rounded_basis = round_d128_1e2(&unrounded_basis);

                                mvmt.cost_basis_lk.set(-rounded_basis);
                                mvmt.proceeds_lk.set(rounded_basis);
                            }
                        }
                    }
                }
            }
        }

        TxType::Flow => {

            if txn.action_record_idx_vec.len() == 2 {

                // Consider asserting TxHasMargin::OneAR

                for ar_num in txn.action_record_idx_vec.iter() {

                    let ar = ars.get(ar_num).unwrap();
                    let movements = ar.get_mvmts_in_ar_in_lot_date_order(acct_map, txns_map);

                    let polarity = ar.direction();

                    for mvmt in movements.iter() {

                        match polarity {

                            Polarity::Outgoing => {
                                // Do nothing.
                                // If 'spot' acct outgoing, the loss nets to the basis in the spent coins.
                                // If margin acct outgoing, no gain, and the incoming gets no cost basis.
                            }

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

        TxType::ToSelf => {
            // Like-kind "exchange," so do nothing.
        }
    }

    Ok(())
}
