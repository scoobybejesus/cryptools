// Copyright (c) 2017-2019, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools/blob/master/LEGAL.txt

use std::fs::{OpenOptions};
use std::collections::{HashMap};
use std::path::PathBuf;
use std::error::Error;
use std::io::prelude::Write;

use decimal::d128;

use crate::transaction::{Transaction, ActionRecord};
use crate::account::{Account, RawAccount};
use crate::core_functions::{ImportProcessParameters};


pub fn _1_account_lot_detail_to_txt(
    settings: &ImportProcessParameters,
    raw_acct_map: &HashMap<u16, RawAccount>,
    acct_map: &HashMap<u16, Account>,
    txns_map: &HashMap<u32, Transaction>,
    ars: &HashMap<u32, ActionRecord>,
) -> Result<(), Box<dyn Error>> {

    // =====================================
    // Exchange BTC
    // Account balance: 0.5000000 BTC; Total cost basis: 450.0000000
    // -------------------------
    // Lot 1
    //     • Σ: 0E-7, with remaining cost basis of 0E-7 and basis date of 2016-01-01
    //     Movements:
    //         1.	0.2500000 BTC (Txn #1) Exchange txn on 1/1/16. - FIRST
    //             Proceeds: 0.0; Cost basis: 220.0000000; for Gain/loss: LT 0; Inc.: 0; Exp.: 0.
    //         2.	-0.2500000 BTC (Txn #2) Exchange txn on 2/1/16. - SECOND
    //             Proceeds: 250.0; Cost basis: -220.0; for Gain/loss: ST 30.0; Inc.: 0; Exp.: 0.
    // -------------------------
    // Lot 2
    //     • Σ: 0E-7, with remaining cost basis of 0.0 and basis date of 2016-03-01
    //     Movements:
    //         1.	0.3000000 BTC (Txn #3) Exchange txn on 3/1/16. - THIRD
    //             Proceeds: 0.0; Cost basis: 160.0; for Gain/loss: LT 0; Inc.: 0; Exp.: 0.
    //         2.	-0.3 BTC (Txn #5) Exchange txn on 7/1/16. - FIFTH
    //             Proceeds: 100.0; Cost basis: -160.0; for Gain/loss: ST -60.0; Inc.: 0; Exp.: 0.
    // -------------------------
    // Lot 3
    //     • Σ: 0E-7, with remaining cost basis of 0.0 and basis date of 2016-04-01
    //     Movements:
    //         1.	0.3000000 BTC (Txn #4) Exchange txn on 4/1/16. - FOURTH
    //             Proceeds: 0.0; Cost basis: 210.0; for Gain/loss: LT 0; Inc.: 0; Exp.: 0.
    //         2.	-0.3 BTC (Txn #5) Exchange txn on 7/1/16. - FIFTH
    //             Proceeds: 100.0; Cost basis: -210.0; for Gain/loss: ST -110.0; Inc.: 0; Exp.: 0.
    // -------------------------
    // Lot 4
    //     • Σ: 0.5000000, with remaining cost basis of 450.0 and basis date of 2016-10-01
    //     Movements:
    //         1.	1.0000000 BTC (Txn #6) Exchange txn on 10/1/16. - SIXTH
    //             Proceeds: 0.0; Cost basis: 900.0; for Gain/loss: LT 0; Inc.: 0; Exp.: 0.
    //         2.	-0.5000000 BTC (Txn #7) ToSelf txn on 1/1/18. - SEVENTH
    //             Proceeds: 0.0; Cost basis: -450.0; for Gain/loss: LT 0; Inc.: 0; Exp.: 0.



    let file_name = PathBuf::from("T1_Acct_lot_detail.txt");
    let path = PathBuf::from(&settings.export_path.clone());
    let full_path: PathBuf = [path, file_name].iter().collect();

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(full_path)?;

    let length = acct_map.len();

    writeln!(file, "Account Listing - All Lots - All Movements - with high level of detail.
\nCosting method used: {}.
Home currency: {}
Enable like-kind treatment: {}",
        settings.costing_method,
        settings.home_currency,
        settings.enable_like_kind_treatment
    )?;

    if settings.enable_like_kind_treatment {
        writeln!(file, "Like-kind cut-off date: {}.",
            settings.lk_cutoff_date_string
        )?;
    }

    for j in 1..=length {

        let acct = acct_map.get(&(j as u16)).unwrap();
        let raw_acct = raw_acct_map.get(&acct.raw_key).unwrap();

        if acct.list_of_lots.borrow().len() > 0 {

            writeln!(file, "\n\n=====================================")?;
            writeln!(file, "{} {}", raw_acct.name, raw_acct.ticker)?;
            writeln!(file, "Account balance: {} {}; Total cost basis: {}",
                acct.get_sum_of_amts_in_lots(),
                raw_acct.ticker,
                acct.get_sum_of_lk_basis_in_lots()
            )?;
        } else {
            continue
        }
        if raw_acct.is_margin { writeln!(file, "Margin Account")?; }

        for (lot_idx, lot) in acct.list_of_lots.borrow().iter().enumerate() {

            let lk_lot_basis = lot.get_sum_of_lk_basis_in_lot();
            let movements_sum = lot.get_sum_of_amts_in_lot();

            if acct.list_of_lots.borrow().len() > 0 {

                writeln!(file, "-------------------------")?;
                writeln!(file, "  Lot {}", (lot_idx+1))?;
                writeln!(file, "\t• Σ: {}, with remaining cost basis of {} and basis date of {}",
                    movements_sum,
                    lk_lot_basis,
                    lot.date_for_basis_purposes
                )?;
                writeln!(file, "\t Movements:")?;

                for (m_idx, mvmt) in lot.movements.borrow().iter().enumerate() {

                    let txn = txns_map.get(&mvmt.transaction_key).unwrap();
                    let tx_type = txn.transaction_type(ars, raw_acct_map, acct_map)?;

                    let description_str = format!("\t\t{}.\t{} {} (Txn #{}) {} txn on {}. - {}",
                        (m_idx+1),
                        mvmt.amount,
                        raw_acct.ticker,
                        mvmt.transaction_key,
                        tx_type,
                        mvmt.date_as_string,
                        txn.user_memo
                    );

                    writeln!(file, "{}", description_str)?;

                    let lk_proceeds = mvmt.proceeds_lk.get();
                    let lk_cost_basis = mvmt.cost_basis_lk.get();
                    let gain_loss: d128;

                    // if mvmt.amount > d128!(0) { // Can't have a gain on an incoming txn
                    //     gain_loss = d128!(0)
                    // } else
                    if raw_acct.is_home_currency(&settings.home_currency) {  //  Can't have a gain disposing home currency
                        gain_loss = d128!(0)
                    // } else if tx_type == TxType::ToSelf {   //  Can't have a gain sending to yourself
                    //     gain_loss = d128!(0)
                    } else {
                        gain_loss = lk_proceeds + lk_cost_basis;
                    }

                    let income = mvmt.get_income(ars, raw_acct_map,	acct_map, txns_map)?;
                    let expense = mvmt.get_expense(ars, raw_acct_map, acct_map, txns_map)?;

                    let activity_str = format!("\t\t\tProceeds: {}; Cost basis: {}; for Gain/loss: {} {}; Inc.: {}; Exp.: {}.",
                        lk_proceeds,
                        lk_cost_basis,
                        mvmt.get_term(acct_map, ars),
                        gain_loss,
                        income,
                        expense,
                    );

                    writeln!(file, "{}", activity_str)?;

                    // if settings.enable_like_kind_treatment {
                    //     let dg_prev = mvmt.proceeds_lk.get();
                    //     let dg_curr = mvmt.cost_basis_lk.get();

                    //     let activity_str = format!("\t\t\tGain deferred in this txn: {}; Accumulated in prior txns: {}",
                    //         dg_curr,
                    //         dg_prev,
                    //     );

                    //     writeln!(file, "{}", activity_str)?
                    // }
                }
            }
        }
    }

    Ok(())
}

pub fn _2_account_lot_summary_to_txt(
    settings: &ImportProcessParameters,
    raw_acct_map: &HashMap<u16, RawAccount>,
    acct_map: &HashMap<u16, Account>,
) -> Result<(), Box<dyn Error>> {

// =====================================
// Bank USD
// Account balance: -220.0000000 USD; Total cost basis: -220.0000000
//   Lot 1 • Σ: -220.0000000, with remaining cost basis of -220.0000000 and basis date of 2016-01-01

// =====================================
// Exchange BTC
// Account balance: 0.5000000 BTC; Total cost basis: 450.0000000
//   Lot 1 • Σ: 0E-7, with remaining cost basis of 0E-7 and basis date of 2016-01-01
//   Lot 2 • Σ: 0E-7, with remaining cost basis of 0.0 and basis date of 2016-03-01
//   Lot 3 • Σ: 0E-7, with remaining cost basis of 0.0 and basis date of 2016-04-01
//   Lot 4 • Σ: 0.5000000, with remaining cost basis of 450.0 and basis date of 2016-10-01



    let file_name = PathBuf::from("T2_Acct_lot_summary.txt");
    let path = PathBuf::from(&settings.export_path.clone());
    let full_path: PathBuf = [path, file_name].iter().collect();

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(full_path)?;

    let length = acct_map.len();

    writeln!(file, "Account Listing - All Lots - No Movements - Summary detail.
\nCosting method used: {}.
Home currency: {}
Enable like-kind treatment: {}",
        settings.costing_method,
        settings.home_currency,
        settings.enable_like_kind_treatment
    )?;

    if settings.enable_like_kind_treatment {
        writeln!(file, "Like-kind cut-off date: {}.",
            settings.lk_cutoff_date_string
        )?;
    }

    for j in 1..=length {

        let acct = acct_map.get(&(j as u16)).unwrap();
        let raw_acct = raw_acct_map.get(&acct.raw_key).unwrap();

        if acct.list_of_lots.borrow().len() > 0 {

            writeln!(file, "\n=====================================")?;
            writeln!(file, "{} {}", raw_acct.name, raw_acct.ticker)?;
            writeln!(file, "Account balance: {} {}; Total cost basis: {}",
                acct.get_sum_of_amts_in_lots(),
                raw_acct.ticker,
                acct.get_sum_of_lk_basis_in_lots()
            )?;
        }
        if raw_acct.is_margin { writeln!(file, "Margin Account")?; }

        for (lot_idx, lot) in acct.list_of_lots.borrow().iter().enumerate() {

            let lk_lot_basis = lot.get_sum_of_lk_basis_in_lot();
            let movements_sum = lot.get_sum_of_amts_in_lot();

            if acct.list_of_lots.borrow().len() > 0 {

                writeln!(file, "  Lot {} • Σ: {}, with remaining cost basis of {} and basis date of {}",
                    (lot_idx+1),
                    movements_sum,
                    lk_lot_basis,
                    lot.date_for_basis_purposes
                )?;
            }
        }
    }

    Ok(())
}

pub fn _3_account_lot_summary_non_zero_to_txt(
    settings: &ImportProcessParameters,
    raw_acct_map: &HashMap<u16, RawAccount>,
    acct_map: &HashMap<u16, Account>,
) -> Result<(), Box<dyn Error>> {

// =====================================
// Exchange BTC
// Account balance: 0.5000000 BTC; Total cost basis: 450.0000000
//   Lot 4 • Σ: 0.5000000, with remaining cost basis of 450.0 and basis date of 2016-10-01

// =====================================
// Simplewallet XMR
// Account balance: 400.0000000 XMR; Total cost basis: 2000.0
//   Lot 1 • Σ: 400.0000000, with remaining cost basis of 2000.0 and basis date of 2018-02-01



    let file_name = PathBuf::from("T3_Acct_lot_summary_non_zero.txt");
    let path = PathBuf::from(&settings.export_path.clone());
    let full_path: PathBuf = [path, file_name].iter().collect();

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(full_path)?;

    let length = acct_map.len();

    writeln!(file, "Account Listing - Non-zero Lots - No Movements - Summary detail.
\nCosting method used: {}.
Home currency: {}
Enable like-kind treatment: {}",
        settings.costing_method,
        settings.home_currency,
        settings.enable_like_kind_treatment
    )?;

    if settings.enable_like_kind_treatment {
        writeln!(file, "Like-kind cut-off date: {}.",
            settings.lk_cutoff_date_string
        )?;
    }

    for j in 1..=length {

        let acct = acct_map.get(&(j as u16)).unwrap();
        let raw_acct = raw_acct_map.get(&acct.raw_key).unwrap();
        let amt_in_acct = acct.get_sum_of_amts_in_lots();

        if acct.list_of_lots.borrow().len() > 0 {
            if amt_in_acct > d128!(0) {

                writeln!(file, "\n=====================================")?;
                writeln!(file, "{} {}", raw_acct.name, raw_acct.ticker)?;
                writeln!(file, "Account balance: {} {}; Total cost basis: {}",
                    amt_in_acct,
                    raw_acct.ticker,
                    acct.get_sum_of_lk_basis_in_lots()
                )?;
            } else {
                continue
            }
        }
        if raw_acct.is_margin { writeln!(file, "Margin Account")?; }

        for (lot_idx, lot) in acct.list_of_lots.borrow().iter().enumerate() {

            let lk_lot_basis = lot.get_sum_of_lk_basis_in_lot();
            let movements_sum = lot.get_sum_of_amts_in_lot();

            if acct.list_of_lots.borrow().len() > 0 {
                if movements_sum > d128!(0) {

                    writeln!(file, "  Lot {} • Σ: {}, with remaining cost basis of {} and basis date of {}",
                        (lot_idx+1),
                        movements_sum,
                        lk_lot_basis,
                        lot.date_for_basis_purposes
                    )?;
                }
            }
        }
    }

    Ok(())
}