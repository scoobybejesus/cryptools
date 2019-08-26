// Copyright (c) 2017-2019, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools-rs/LEGAL.txt

use std::rc::{Rc};
use std::fs::File;
use std::collections::{HashMap};
use std::path::PathBuf;

use decimal::d128;

use crate::transaction::{Transaction, ActionRecord, Polarity, TxType};
use crate::account::{Account, RawAccount, Term};
use crate::cli_user_choices::{LotProcessingChoices};


pub fn _1_account_sums_to_csv(
    settings: &LotProcessingChoices,
    raw_acct_map: &HashMap<u16, RawAccount>,
    acct_map: &HashMap<u16, Account>
) {

    let mut rows: Vec<Vec<String>> = [].to_vec();
    let mut header: Vec<String> = [].to_vec();

    header.extend_from_slice(&[
        "Account".to_string(),
        "Balance".to_string(),
        "Ticker".to_string(),
        "Cost Basis".to_string(),
        "Total lots".to_string(),
    ]);
    rows.push(header);

    let length = acct_map.len();

    for j in 1..=length {

        let acct = acct_map.get(&(j as u16)).unwrap();
        let mut row: Vec<String> = [].to_vec();

        let balance: String;
        let tentative_balance = acct.get_sum_of_amts_in_lots();

        if tentative_balance == d128!(0) {
            balance = "0.00".to_string()
        } else { balance = tentative_balance.to_string() }

        let raw_acct = raw_acct_map.get(&acct.raw_key).unwrap();
        let cost_basis: String;

        if raw_acct.is_margin { cost_basis = "0.00".to_string() } else {
            let tentative_cost_basis = acct.get_sum_of_basis_in_lots();
            if tentative_cost_basis == d128!(0) {
                cost_basis = "0.00".to_string()
            } else { cost_basis = tentative_cost_basis.to_string() }
        }

        row.push(raw_acct.name.to_string());
        row.push(balance);
        row.push(raw_acct.ticker.to_string());
        row.push(cost_basis);
        row.push(acct.list_of_lots.borrow().len().to_string());
        rows.push(row);
    }
    let file_name = PathBuf::from("1_Acct_Sum_with_cost_basis.csv");
    let path = PathBuf::from(&settings.export_path.clone());

    let full_path: PathBuf = [path, file_name].iter().collect();
    let buffer = File::create(full_path).unwrap();
    let mut wtr = csv::Writer::from_writer(buffer);

    for row in rows.iter() {
        wtr.write_record(row).expect("Could not write row to CSV file");
    }
    wtr.flush().expect("Could not flush Writer, though file should exist and be complete");
}

pub fn _2_account_sums_nonzero_to_csv(
    acct_map: &HashMap<u16, Account>,
    settings: &LotProcessingChoices,
    raw_acct_map: &HashMap<u16, RawAccount>
) {

    let mut rows: Vec<Vec<String>> = [].to_vec();
    let mut header: Vec<String> = [].to_vec();

    header.extend_from_slice(&[
        "Account".to_string(),
        "Balance".to_string(),
        "Ticker".to_string(),
        "Cost basis".to_string(),
        "Total lots".to_string(),
    ]);
    rows.push(header);

    let length = acct_map.len();

    for j in 1..=length {

        let acct = acct_map.get(&(j as u16)).unwrap();
        let mut row: Vec<String> = [].to_vec();

        let raw_acct = raw_acct_map.get(&acct.raw_key).unwrap();
        let name = raw_acct.name.to_string();

        let balance: String;
        let mut balance_d128 = d128!(0);
        let tentative_balance = acct.get_sum_of_amts_in_lots();

        if tentative_balance == d128!(0) {
            balance = "0.00".to_string()
        } else { balance_d128 += tentative_balance; balance = tentative_balance.to_string() }

        let cost_basis: String;

        if raw_acct.is_margin { cost_basis = "0.00".to_string() } else {
            let tentative_cost_basis = acct.get_sum_of_basis_in_lots();
            if tentative_cost_basis == d128!(0) {
                cost_basis = "0.00".to_string()
            } else { cost_basis = tentative_cost_basis.to_string() }
        }

        if balance_d128 != d128!(0) {
            row.push(name);
            row.push(balance);
            row.push(raw_acct.ticker.to_string());
            row.push(cost_basis);
            row.push(acct.list_of_lots.borrow().len().to_string());
            rows.push(row);
        }
    }

    let file_name = PathBuf::from("2_Acct_Sum_with_nonzero_cost_basis.csv");
    let path = PathBuf::from(&settings.export_path.clone());

    let full_path: PathBuf = [path, file_name].iter().collect();
    let buffer = File::create(full_path).unwrap();
    let mut wtr = csv::Writer::from_writer(buffer);

    for row in rows.iter() {
        wtr.write_record(row).expect("Could not write row to CSV file");
    }
    wtr.flush().expect("Could not flush Writer, though file should exist and be complete");
}

// pub fn transactions_to_csv(
//     transactions: &[Rc<Transaction>],
//     ars: &HashMap<u32, ActionRecord>,
//     raw_acct_map: &HashMap<u16, RawAccount>,
//     acct_map: &HashMap<u16, Account>,
//     txns_map: &HashMap<u32, Transaction>,) {

//     let mut rows: Vec<Vec<String>> = [].to_vec();
//     let mut header: Vec<String> = [].to_vec();
//     header.extend_from_slice(&[
//         "Date".to_string(),
//         "Txn#".to_string(),
//         "Type".to_string(),
//         "Memo".to_string(),
//         "Amount".to_string(),
//         "Ticker".to_string(),
//         "Proceeds".to_string(),
//         "Cost basis".to_string(),
//         "Gain/loss".to_string(),
//         "Term".to_string(),
//         "Income".to_string(),
//         "Expense".to_string(),
//     ]);
//     rows.push(header);
//     for txn in transactions {
//         for mvmt in txn.flow_or_outgoing_exchange_movements.borrow().iter() {
//             let lot = mvmt.borrow().get_lot(acct_map, ars);
//             let acct = acct_map.get(&lot.account_key).unwrap();
//             let raw_acct = raw_acct_map.get(&acct.raw_key).unwrap();
//             let mut row: Vec<String> = [].to_vec();
//             row.push(txn.date.format("%Y/%m/%d").to_string());
//             row.push(txn.tx_number.to_string());
//             row.push(txn.transaction_type(&ars, &raw_acct_map, &acct_map).to_string());
//             row.push(txn.memo.to_string());
//             row.push(mvmt.borrow().amount.to_string());
//             row.push(raw_acct.ticker.to_string());
//             row.push(mvmt.borrow().proceeds.to_string());
//             row.push(mvmt.borrow().cost_basis.to_string());
//             row.push(mvmt.borrow().get_gain_or_loss().to_string());
//             row.push(mvmt.borrow().get_term(acct_map, ars).to_string());
//             row.push(mvmt.borrow().get_income(ars, &raw_acct_map, &acct_map, &txns_map).to_string());
//             row.push(mvmt.borrow().get_expense(ars, &raw_acct_map, &acct_map, &txns_map).to_string());
//             rows.push(row);
//         }
//     }
//     let buffer = File::create("/Users/scoob/Documents/Testing/rust_exports/test/txns-3rd-try.csv").unwrap();
//     let mut wtr = csv::Writer::from_writer(buffer);
//     for row in rows.iter() {
//         wtr.write_record(row).expect("Could not write row to CSV file");
//     }
//     wtr.flush().expect("Could not flush Writer, though file should exist and be complete");
// }

pub fn _5_transaction_mvmt_summaries_to_csv(
    settings: &LotProcessingChoices,
    ars: &HashMap<u32, ActionRecord>,
    raw_acct_map: &HashMap<u16, RawAccount>,
    acct_map: &HashMap<u16, Account>,
    txns_map: &HashMap<u32, Transaction>,
) {

    let mut rows: Vec<Vec<String>> = [].to_vec();
    let mut header: Vec<String> = [].to_vec();

    header.extend_from_slice(&[
        "Date".to_string(),
        "Txn#".to_string(),
        "Type".to_string(),
        "Memo".to_string(),
        "Amount".to_string(),
        "Ticker".to_string(),
        "Term".to_string(),
        "Proceeds".to_string(),
        "Cost basis".to_string(),
        "Gain/loss".to_string(),
        "Income".to_string(),
        "Expense".to_string(),
    ]);
    rows.push(header);

    let length = txns_map.len();

    for txn_num in 1..=length {

        let txn_num = txn_num as u32;
        let txn = txns_map.get(&(txn_num)).unwrap();
        let txn_date_string = txn.date.format("%Y/%m/%d").to_string();
        let tx_num_string = txn.tx_number.to_string();
        let tx_type_string = txn.transaction_type(ars, &raw_acct_map, &acct_map).to_string();
        let tx_memo_string = txn.memo.to_string();
        let mut term_st: Option<Term> = None;
        let mut term_lt: Option<Term> = None;
        let mut ticker: Option<String> = None;
        let mut polarity: Option<Polarity> = None;

        let mut amount_st = d128!(0);
        let mut proceeds_st = d128!(0);
        let mut cost_basis_st = d128!(0);

        let mut income_st = d128!(0);
        let mut expense_st = d128!(0);

        let mut amount_lt = d128!(0);
        let mut proceeds_lt = d128!(0);
        let mut cost_basis_lt = d128!(0);

        let mut income_lt = d128!(0);
        let mut expense_lt = d128!(0);

        let flow_or_outgoing_exchange_movements = txn.get_outgoing_exchange_and_flow_mvmts(
            settings,
            ars,
            raw_acct_map,
            acct_map,
            txns_map
        );

        for mvmt in flow_or_outgoing_exchange_movements.iter() {
            let lot = mvmt.get_lot(acct_map, ars);
            let acct = acct_map.get(&lot.account_key).unwrap();
            let raw_acct = raw_acct_map.get(&acct.raw_key).unwrap();

            if let None = ticker { ticker = Some(raw_acct.ticker.clone()) };

            if let None = polarity {
                polarity = if mvmt.amount > d128!(0) {
                    Some(Polarity::Incoming)
                    } else { Some(Polarity::Outgoing)
                };
            }

            let term = mvmt.get_term(acct_map, ars);

            if term == Term::LT {
                amount_lt += mvmt.amount;
                proceeds_lt += mvmt.proceeds.get();
                cost_basis_lt += mvmt.cost_basis.get();
                match term_lt {
                    None => { term_lt = Some(term)}
                    _ => {}
                }
            } else {
                assert_eq!(term, Term::ST);
                amount_st += mvmt.amount;
                proceeds_st += mvmt.proceeds.get();
                cost_basis_st += mvmt.cost_basis.get();
                if term_st == None {
                    term_st = Some(term);
                }
            }
        }

        if (txn.transaction_type(ars, &raw_acct_map, &acct_map) == TxType::Flow) & (polarity == Some(Polarity::Incoming)) {
            // println!("Incoming flow {}", txn.tx_number);
            income_st = proceeds_st;
            proceeds_st = d128!(0);
            cost_basis_st = d128!(0);
            income_lt = proceeds_lt;
            proceeds_lt = d128!(0);
            cost_basis_lt = d128!(0);
        }

        if (txn.transaction_type(ars, &raw_acct_map, &acct_map) == TxType::Flow) & (polarity == Some(Polarity::Outgoing)) {
            // println!("Outgoing flow {}, proceeds_st {}, proceeds_lt {}", txn.tx_number, proceeds_st, proceeds_lt);
            expense_st -= proceeds_st;
            expense_lt -= proceeds_lt;
        }

        if let Some(term) = term_st {

            let mut row: Vec<String> = [].to_vec();

            row.push(txn_date_string.clone());
            row.push(tx_num_string.clone());
            row.push(tx_type_string.clone());
            row.push(tx_memo_string.clone());
            row.push(amount_st.to_string());
            row.push(ticker.clone().unwrap());
            row.push(term.abbr_string());
            row.push(proceeds_st.to_string());
            row.push(cost_basis_st.to_string());
            row.push((proceeds_st + cost_basis_st).to_string());
            row.push(income_st.to_string());
            row.push(expense_st.to_string());

            rows.push(row);
        }
        if let Some(term) = term_lt {

            let mut row: Vec<String> = [].to_vec();

            row.push(txn_date_string);
            row.push(tx_num_string);
            row.push(tx_type_string);
            row.push(tx_memo_string);
            row.push(amount_lt.to_string());
            row.push(ticker.unwrap());
            row.push(term.abbr_string());
            row.push(proceeds_lt.to_string());
            row.push(cost_basis_lt.to_string());
            row.push((proceeds_lt + cost_basis_lt).to_string());
            row.push(income_lt.to_string());
            row.push(expense_lt.to_string());

            rows.push(row);
        }
    }

    let file_name = PathBuf::from("5_Txns_mvmts_summary.csv");
    let path = PathBuf::from(&settings.export_path);

    let full_path: PathBuf = [path, file_name].iter().collect();
    let buffer = File::create(full_path).unwrap();
    let mut wtr = csv::Writer::from_writer(buffer);

    for row in rows.iter() {
        wtr.write_record(row).expect("Could not write row to CSV file");
    }
    wtr.flush().expect("Could not flush Writer, though file should exist and be complete");
}

pub fn accounts_to_csv(
    accounts: &[Rc<Account>],
    ars: &HashMap<u32, ActionRecord>,
    raw_acct_map: &HashMap<u16, RawAccount>,
    acct_map: &HashMap<u16, Account>,
    txns_map: &HashMap<u32, Transaction>,
) {

    let mut rows: Vec<Vec<String>> = [].to_vec();
    let mut header: Vec<String> = [].to_vec();

    header.extend_from_slice(&[
        "#".to_string(),
        "Account".to_string(),
        "Ticker".to_string(),
        "Margin".to_string(),
        "Date".to_string(),
        "Txn#".to_string(),
        "Type".to_string(),
        "Memo".to_string(),
        "Amount".to_string(),
        "Proceeds".to_string(),
        "Cost basis\n".to_string(),
        "Gain/loss".to_string(),
        "Term".to_string(),
        "Income".to_string(),
        "Expense".to_string(),
    ]);
    rows.push(header);

    for acct in accounts {
        for lot in acct.list_of_lots.borrow().iter() {
            for mvmt in lot.movements.borrow().iter() {

                let raw_acct = raw_acct_map.get(&acct.raw_key).unwrap();
                let txn = txns_map.get(&mvmt.transaction_key).unwrap();
                let mut row: Vec<String> = [].to_vec();

                row.push(raw_acct.account_num.to_string());
                row.push(raw_acct.name.to_string());
                row.push(raw_acct.ticker.to_string());
                row.push(raw_acct.is_margin.to_string());
                row.push(mvmt.date.format("%Y/%m/%d").to_string());
                row.push(txn.tx_number.to_string());
                row.push(txn.transaction_type(ars, &raw_acct_map, &acct_map).to_string());
                row.push(txn.memo.to_string());
                row.push(mvmt.amount.to_string());
                row.push(mvmt.proceeds.get().to_string());
                row.push(mvmt.cost_basis.get().to_string());
                row.push(mvmt.get_gain_or_loss().to_string());
                row.push(mvmt.get_term(acct_map, ars).to_string());
                row.push(mvmt.get_income(ars, &raw_acct_map, &acct_map, &txns_map).to_string());
                row.push(mvmt.get_expense(ars, &raw_acct_map, &acct_map, &txns_map).to_string());

                rows.push(row);
            }
        }
    }

    let buffer = File::create("/Users/scoob/Documents/Testing/rust_exports/test/accts-1st-try.csv").unwrap();
    let mut wtr = csv::Writer::from_writer(buffer);

    for row in rows.iter() {
        wtr.write_record(row).expect("Could not write row to CSV file");
    }
    wtr.flush().expect("Could not flush Writer, though file should exist and be complete");
}
