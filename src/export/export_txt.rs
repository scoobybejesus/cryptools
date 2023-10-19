// Copyright (c) 2017-2023, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools/blob/master/LEGAL.txt

use std::fs::OpenOptions;
use std::collections::HashMap;
use std::path::PathBuf;
use std::error::Error;
use std::io::prelude::Write;

use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crptls::transaction::{Transaction, ActionRecord};
use crptls::account::{Account, RawAccount};
use crptls::core_functions::ImportProcessParameters;


pub fn _1_account_lot_detail_to_txt(
    settings: &ImportProcessParameters,
    raw_acct_map: &HashMap<u16, RawAccount>,
    acct_map: &HashMap<u16, Account>,
    ars: &HashMap<u32, ActionRecord>,
    txns_map: &HashMap<u32, Transaction>,
) -> Result<(), Box<dyn Error>> {

    // =====================================
    // Exchange BTC
    // Account balance: 0.5000000 BTC; Total cost basis: 450.0000000
    // -------------------------
    // Lot 1
    //     • Σ: 0.00, with remaining cost basis of 0.00 and basis date of 2016-01-01
    //     Movements:
    //         1.	0.2500000 BTC (Txn #1) Exchange txn on 2016-01-01. - FIRST
    //             Proceeds: 0.0; Cost basis: 220.0000000; for Gain/loss: LT 0; Inc.: 0; Exp.: 0.
    //         2.	-0.2500000 BTC (Txn #2) Exchange txn on 2016-02-01. - SECOND
    //             Proceeds: 250.0; Cost basis: -220.0; for Gain/loss: ST 30.0; Inc.: 0; Exp.: 0.
    // -------------------------
    // Lot 2
    //     • Σ: 0.00, with remaining cost basis of 0.00 and basis date of 2016-03-01
    //     Movements:
    //         1.	0.3000000 BTC (Txn #3) Exchange txn on 2016-03-01. - THIRD
    //             Proceeds: 0.0; Cost basis: 160.0; for Gain/loss: LT 0; Inc.: 0; Exp.: 0.
    //         2.	-0.3 BTC (Txn #5) Exchange txn on 2016-07-01. - FIFTH
    //             Proceeds: 100.0; Cost basis: -160.0; for Gain/loss: ST -60.0; Inc.: 0; Exp.: 0.
    // -------------------------
    // Lot 3
    //     • Σ: 0.00, with remaining cost basis of 0.00 and basis date of 2016-04-01
    //     Movements:
    //         1.	0.3000000 BTC (Txn #4) Exchange txn on 2016-04-01. - FOURTH
    //             Proceeds: 0.0; Cost basis: 210.0; for Gain/loss: LT 0; Inc.: 0; Exp.: 0.
    //         2.	-0.3 BTC (Txn #5) Exchange txn on 2016-07-01. - FIFTH
    //             Proceeds: 100.0; Cost basis: -210.0; for Gain/loss: ST -110.0; Inc.: 0; Exp.: 0.
    // -------------------------
    // Lot 4
    //     • Σ: 0.5000000, with remaining cost basis of 450.0 and basis date of 2016-10-01
    //     Movements:
    //         1.	1.0000000 BTC (Txn #6) Exchange txn on 2016-10-01. - SIXTH
    //             Proceeds: 0.0; Cost basis: 900.0; for Gain/loss: LT 0; Inc.: 0; Exp.: 0.
    //         2.	-0.5000000 BTC (Txn #7) ToSelf txn on 2018-01-01. - SEVENTH
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

    let home_currency = &settings.home_currency;

    writeln!(file, "Account Listing - All Lots - All Movements - with high level of detail.
\nCosting method used: {}.
Home currency: {}
Enable like-kind treatment: {}",
        settings.costing_method,
        home_currency,
        settings.lk_treatment_enabled
    )?;

    if settings.lk_treatment_enabled {
        writeln!(file, "Like-kind cut-off date: {}.",
            settings.lk_cutoff_date
        )?;
    }

    for j in 1..=length {

        let acct = acct_map.get(&(j as u16)).unwrap();
        let raw_acct = raw_acct_map.get(&acct.raw_key).unwrap();
        let ticker = &raw_acct.ticker;

        if acct.list_of_lots.borrow().len() > 0 {

            writeln!(file, "\n\n=====================================")?;
            writeln!(file, "{} {}", raw_acct.name, ticker)?;

            let acct_bal_line;

            if raw_acct.is_home_currency(home_currency) {
                acct_bal_line = format!("Account balance: {:.2} {}; Total cost basis: {:.2}",
                    acct.get_sum_of_amts_in_lots().to_string().as_str().parse::<f32>()?,
                    ticker,
                    acct.get_sum_of_lk_basis_in_lots().to_string().as_str().parse::<f32>()?
                );
            } else {
                acct_bal_line = format!("Account balance: {} {}; Total cost basis: {:.2}",
                    acct.get_sum_of_amts_in_lots(),
                    ticker,
                    acct.get_sum_of_lk_basis_in_lots().to_string().as_str().parse::<f32>()?
                );
            }

            writeln!(file, "{}", acct_bal_line)?;

        } else {
            continue
        }

        if raw_acct.is_margin { writeln!(file, "Margin Account")?; }

        for (lot_idx, lot) in acct.list_of_lots.borrow().iter().enumerate() {

            let lk_lot_basis = lot.get_sum_of_lk_basis_in_lot();

            let formatted_basis: String;
            if lk_lot_basis == dec!(0) {
                formatted_basis = "0.00".to_string()
            } else { formatted_basis = lk_lot_basis.to_string() }

            let movements_sum = lot.get_sum_of_amts_in_lot();

            let formatted_sum: String;
            if movements_sum == dec!(0) {
                formatted_sum = "0.00".to_string()
            } else { formatted_sum = movements_sum.to_string() }

            if acct.list_of_lots.borrow().len() > 0 {

                writeln!(file, "-------------------------")?;
                writeln!(file, "  Lot {}", (lot_idx+1))?;

                let lot_sum_row;

                if raw_acct.is_home_currency(home_currency) {
                    lot_sum_row = format!("    • Σ: {:.2} {}, with remaining cost basis of {:.2} {} and basis date of {}",
                        formatted_sum.to_string().as_str().parse::<f32>()?,
                        ticker,
                        formatted_basis.to_string().as_str().parse::<f32>()?,
                        home_currency,
                        lot.date_for_basis_purposes
                    )
                } else {
                    lot_sum_row = format!("    • Σ: {} {}, with remaining cost basis of {:.2} {} and basis date of {}",
                        formatted_sum,
                        ticker,
                        formatted_basis.to_string().as_str().parse::<f32>()?,
                        home_currency,
                        lot.date_for_basis_purposes
                    )
                }
                writeln!(file, "{}", lot_sum_row)?;
                writeln!(file, "     Movements:")?;

                for (m_idx, mvmt) in lot.movements.borrow().iter().enumerate() {

                    let txn = txns_map.get(&mvmt.transaction_key).unwrap();
                    let tx_type = txn.transaction_type(ars, raw_acct_map, acct_map)?;

                    let description_string: String;

                    if raw_acct.is_home_currency(home_currency) {
                        description_string = format!("\t{}.  {:<8.2} {} (Txn #{:>4}) {:>9} txn on {:10}. - {}",
                            (m_idx+1),
                            mvmt.amount.to_string().as_str().parse::<f32>()?,
                            ticker,
                            mvmt.transaction_key,
                            tx_type,
                            mvmt.date,
                            txn.user_memo
                        );
                    } else {
                        description_string = format!("\t{}.  {:<8} {} (Txn #{:>4}) {:>9} txn on {:10}. - {}",
                            (m_idx+1),
                            mvmt.amount,
                            ticker,
                            mvmt.transaction_key,
                            tx_type,
                            mvmt.date,
                            txn.user_memo
                        );
                    };

                    writeln!(file, "{}", description_string)?;

                    let lk_proceeds = mvmt.proceeds_lk.get();
                    let lk_cost_basis = mvmt.cost_basis_lk.get();
                    let gain_loss: Decimal;

                    // if mvmt.amount > dec!(0) { // Can't have a gain on an incoming txn
                    //     gain_loss = dec!(0)
                    // } else
                    if raw_acct.is_home_currency(home_currency) {  //  Can't have a gain disposing home currency
                        gain_loss = dec!(0)
                    // } else if tx_type == TxType::ToSelf {   //  Can't have a gain sending to yourself
                    //     gain_loss = dec!(0)
                    } else {
                        gain_loss = lk_proceeds + lk_cost_basis;
                    }

                    let income = mvmt.get_income(ars, raw_acct_map,	acct_map, txns_map)?;
                    let expense = mvmt.get_expense(ars, raw_acct_map, acct_map, txns_map)?;

                    let activity_str = format!("\t    Proceeds: {:>10.2}; Cost basis: {:>10.2}; for Gain/loss: {} {:>10.2}; Inc.: {:>10.2}; Exp.: {:>10.2}.",
                        lk_proceeds.to_string().as_str().parse::<f32>()?,
                        lk_cost_basis.to_string().as_str().parse::<f32>()?,
                        mvmt.get_term(acct_map, ars, txns_map),
                        gain_loss.to_string().as_str().parse::<f32>()?,
                        income.to_string().as_str().parse::<f32>()?,
                        expense.to_string().as_str().parse::<f32>()?,
                    );

                    writeln!(file, "{}", activity_str)?;

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
//   Lot 1 created 2016-01-01 w/ basis date 2016-01-01 • Σ: -220.0000000, and cost basis of -220.0000000

// =====================================
// Exchange BTC
// Account balance: 0.5000000 BTC; Total cost basis: 450.0000000
//   Lot 1 created 2016-01-01 w/ basis date 2016-01-01 • Σ: 0.00, and cost basis of 0.00
//   Lot 2 created 2016-03-01 w/ basis date 2016-03-01 • Σ: 0.00, and cost basis of 0.00
//   Lot 3 created 2016-04-01 w/ basis date 2016-04-01 • Σ: 0.00, and cost basis of 0.00
//   Lot 4 created 2016-10-01 w/ basis date 2016-10-01 • Σ: 0.5000000, and cost basis of 450.00



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
        settings.lk_treatment_enabled
    )?;

    if settings.lk_treatment_enabled {
        writeln!(file, "Like-kind cut-off date: {}.",
            settings.lk_cutoff_date
        )?;
    }

    for j in 1..=length {

        let acct = acct_map.get(&(j as u16)).unwrap();
        let raw_acct = raw_acct_map.get(&acct.raw_key).unwrap();

        if acct.list_of_lots.borrow().len() > 0 {

            writeln!(file, "\n=====================================")?;
            writeln!(file, "{} {}", raw_acct.name, raw_acct.ticker)?;
            writeln!(file, "Account balance: {} {}; Total cost basis: {:.2}",
                acct.get_sum_of_amts_in_lots(),
                raw_acct.ticker,
                acct.get_sum_of_lk_basis_in_lots().to_string().as_str().parse::<f32>()?
            )?;
        }
        if raw_acct.is_margin { writeln!(file, "Margin Account")?; }

        for (lot_idx, lot) in acct.list_of_lots.borrow().iter().enumerate() {

            let lk_lot_basis = lot.get_sum_of_lk_basis_in_lot();

            let formatted_basis: String;
            if lk_lot_basis == dec!(0) {
                formatted_basis = "0.00".to_string()
            } else { formatted_basis = lk_lot_basis.to_string() }

            let movements_sum = lot.get_sum_of_amts_in_lot();

            let formatted_sum: String;
            if movements_sum == dec!(0) {
                formatted_sum = "0.00".to_string()
            } else { formatted_sum = movements_sum.to_string() }

            if acct.list_of_lots.borrow().len() > 0 {

                writeln!(file, "  Lot {:>3} created {} w/ basis date {} • Σ: {:>12}, and cost basis of {:>10.2}",
                    (lot_idx+1),
                    lot.date_of_first_mvmt_in_lot,
                    lot.date_for_basis_purposes,
                    formatted_sum,
                    formatted_basis.to_string().as_str().parse::<f32>()?,
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
//   Lot 4 created 2016-10-01 w/ basis date 2016-10-01 • Σ: 0.5000000, and cost basis of 450.00

// =====================================
// Simplewallet XMR
// Account balance: 400.0000000 XMR; Total cost basis: 2000.0
//   Lot 1 created 2018-02-01 w/ basis date 2018-02-01 • Σ: 400.0000000, and cost basis of 2000.00



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
        settings.lk_treatment_enabled
    )?;

    if settings.lk_treatment_enabled {
        writeln!(file, "Like-kind cut-off date: {}.",
            settings.lk_cutoff_date
        )?;
    }

    for j in 1..=length {

        let acct = acct_map.get(&(j as u16)).unwrap();
        let raw_acct = raw_acct_map.get(&acct.raw_key).unwrap();
        let amt_in_acct = acct.get_sum_of_amts_in_lots();

        if acct.list_of_lots.borrow().len() > 0 {
            if amt_in_acct > dec!(0) {

                writeln!(file, "\n=====================================")?;
                writeln!(file, "{} {}", raw_acct.name, raw_acct.ticker)?;
                writeln!(file, "Account balance: {} {}; Total cost basis: {:.2}",
                    amt_in_acct,
                    raw_acct.ticker,
                    acct.get_sum_of_lk_basis_in_lots().to_string().as_str().parse::<f32>()?
                )?;
            } else {
                continue
            }
        }
        if raw_acct.is_margin { writeln!(file, "Margin Account")?; }

        for (lot_idx, lot) in acct.list_of_lots.borrow().iter().enumerate() {

            let lk_lot_basis = lot.get_sum_of_lk_basis_in_lot();

            let formatted_basis: String;
            if lk_lot_basis == dec!(0) {
                formatted_basis = "0.00".to_string()
            } else { formatted_basis = lk_lot_basis.to_string() }

            let movements_sum = lot.get_sum_of_amts_in_lot();

            if acct.list_of_lots.borrow().len() > 0 && movements_sum > dec!(0) {

                writeln!(file, "  Lot {:>3} created {} w/ basis date {} • Σ: {:>12}, and cost basis of {:>10.2}",
                    (lot_idx+1),
                    lot.date_of_first_mvmt_in_lot,
                    lot.date_for_basis_purposes,
                    movements_sum,
                    formatted_basis.to_string().as_str().parse::<f32>()?,
                )?;
            }
        }
    }

    Ok(())
}


