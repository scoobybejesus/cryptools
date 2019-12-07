// Copyright (c) 2017-2019, scoobybejesus;
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools/blob/master/LEGAL.txt

use std::fs::File;
use std::collections::{HashMap};
use std::path::PathBuf;
use std::error::Error;

use decimal::d128;

use crptls::transaction::{Transaction, ActionRecord, Polarity, TxType};
use crptls::account::{Account, RawAccount, Term};
use crptls::core_functions::{ImportProcessParameters};


pub fn _1_account_sums_to_csv(
    settings: &ImportProcessParameters,
    raw_acct_map: &HashMap<u16, RawAccount>,
    acct_map: &HashMap<u16, Account>
) {

    let mut rows: Vec<Vec<String>> = Vec::with_capacity(acct_map.len());

    let columns = [
        "Account".to_string(),
        "Balance".to_string(),
        "Ticker".to_string(),
        "Cost Basis".to_string(),
        "Total lots".to_string(),
        "Nonzero lots".to_string(),
    ];

    let total_columns = columns.len();
    let mut header: Vec<String> = Vec::with_capacity(total_columns);

    header.extend_from_slice(&columns);
    rows.push(header);

    let length = acct_map.len();

    for j in 1..=length {

        let acct = acct_map.get(&(j as u16)).unwrap();
        let mut row: Vec<String> = Vec::with_capacity(total_columns);

        let balance: String;
        let tentative_balance = acct.get_sum_of_amts_in_lots();

        if tentative_balance == d128!(0) {
            balance = "0.00".to_string()
        } else { balance = tentative_balance.to_string() }

        let raw_acct = raw_acct_map.get(&acct.raw_key).unwrap();
        let lk_cost_basis: String;

        if raw_acct.is_margin { lk_cost_basis = "0.00".to_string() } else {
            let tentative_lk_cost_basis = acct.get_sum_of_lk_basis_in_lots();
            if tentative_lk_cost_basis == d128!(0) {
                lk_cost_basis = "0.00".to_string()
            } else { lk_cost_basis = tentative_lk_cost_basis.to_string() }
        }

        let cb_f32 = lk_cost_basis.as_str().parse::<f32>().unwrap();
        let cb = format!("{:.2}", cb_f32);

        let nonzero_lots = acct.get_num_of_nonzero_lots();

        row.push(raw_acct.name.to_string());
        row.push(balance);
        row.push(raw_acct.ticker.to_string());
        row.push(cb);
        row.push(acct.list_of_lots.borrow().len().to_string());
        row.push(nonzero_lots.to_string());
        rows.push(row);
    }
    let file_name = PathBuf::from("C1_Acct_Sum_with_cost_basis.csv");
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
    settings: &ImportProcessParameters,
    raw_acct_map: &HashMap<u16, RawAccount>,
    acct_map: &HashMap<u16, Account>,
) {

    let mut rows: Vec<Vec<String>> = Vec::with_capacity(acct_map.len());    //  more than needed...

    let columns = [
        "Account".to_string(),
        "Balance".to_string(),
        "Ticker".to_string(),
        "Cost basis".to_string(),
        "Total lots".to_string(),
        "Nonzero lots".to_string(),
    ];

    let total_columns = columns.len();
    let mut header: Vec<String> = Vec::with_capacity(total_columns);

    header.extend_from_slice(&columns);
    rows.push(header);

    let length = acct_map.len();

    for j in 1..=length {

        let acct = acct_map.get(&(j as u16)).unwrap();
        let mut row: Vec<String> = Vec::with_capacity(total_columns);

        let raw_acct = raw_acct_map.get(&acct.raw_key).unwrap();
        let name = raw_acct.name.to_string();

        let balance: String;
        let mut balance_d128 = d128!(0);
        let tentative_balance = acct.get_sum_of_amts_in_lots();

        if tentative_balance == d128!(0) {
            balance = "0.00".to_string()
        } else { balance_d128 += tentative_balance; balance = tentative_balance.to_string() }

        let lk_cost_basis: String;

        if raw_acct.is_margin { lk_cost_basis = "0.00".to_string() } else {
            let tentative_lk_cost_basis = acct.get_sum_of_lk_basis_in_lots();
            if tentative_lk_cost_basis == d128!(0) {
                lk_cost_basis = "0.00".to_string()
            } else { lk_cost_basis = tentative_lk_cost_basis.to_string() }
        }

        let cb_f32 = lk_cost_basis.as_str().parse::<f32>().unwrap();
        let cb = format!("{:.2}", cb_f32);

        let nonzero_lots = acct.get_num_of_nonzero_lots();

        if balance_d128 != d128!(0) {
            row.push(name);
            row.push(balance);
            row.push(raw_acct.ticker.to_string());
            row.push(cb);
            row.push(acct.list_of_lots.borrow().len().to_string());
            row.push(nonzero_lots.to_string());
            rows.push(row);
        }
    }

    let file_name = PathBuf::from("C2_Acct_Sum_with_nonzero_cost_basis.csv");
    let path = PathBuf::from(&settings.export_path.clone());

    let full_path: PathBuf = [path, file_name].iter().collect();
    let buffer = File::create(full_path).unwrap();
    let mut wtr = csv::Writer::from_writer(buffer);

    for row in rows.iter() {
        wtr.write_record(row).expect("Could not write row to CSV file");
    }
    wtr.flush().expect("Could not flush Writer, though file should exist and be complete");
}

pub fn _3_account_sums_to_csv_with_orig_basis(
    settings: &ImportProcessParameters,
    raw_acct_map: &HashMap<u16, RawAccount>,
    acct_map: &HashMap<u16, Account>
) {

    let mut rows: Vec<Vec<String>> = Vec::with_capacity(acct_map.len());

    let columns = [
        "Account".to_string(),
        "Balance".to_string(),
        "Ticker".to_string(),
        "Orig. Cost Basis".to_string(),
        "LK Cost Basis".to_string(),
        "Total lots".to_string(),
        "Nonzero lots".to_string(),
    ];

    let total_columns = columns.len();
    let mut header: Vec<String> = Vec::with_capacity(total_columns);

    header.extend_from_slice(&columns);
    rows.push(header);

    let length = acct_map.len();

    for j in 1..=length {

        let acct = acct_map.get(&(j as u16)).unwrap();
        let mut row: Vec<String> = Vec::with_capacity(6);

        let balance: String;
        let tentative_balance = acct.get_sum_of_amts_in_lots();

        if tentative_balance == d128!(0) {
            balance = "0.00".to_string()
        } else { balance = tentative_balance.to_string() }

        let raw_acct = raw_acct_map.get(&acct.raw_key).unwrap();
        let lk_cost_basis: String;
        let orig_cost_basis: String;

        if raw_acct.is_margin {

            lk_cost_basis = "0.00".to_string();
            orig_cost_basis = "0.00".to_string();
        } else {

            let tentative_lk_cost_basis = acct.get_sum_of_lk_basis_in_lots();
            let tentative_orig_cost_basis = acct.get_sum_of_orig_basis_in_lots();

            if tentative_lk_cost_basis == d128!(0) {
                lk_cost_basis = "0.00".to_string()
            } else { lk_cost_basis = tentative_lk_cost_basis.to_string() }

            if tentative_orig_cost_basis == d128!(0) {
                orig_cost_basis = "0.00".to_string()
            } else { orig_cost_basis = tentative_orig_cost_basis.to_string() }
        }

        let cb_f32 = lk_cost_basis.to_string().as_str().parse::<f32>().unwrap();
        let cb = format!("{:.2}", cb_f32);

        let ocb_f32 = orig_cost_basis.as_str().parse::<f32>().unwrap();
        let ocb = format!("{:.2}", ocb_f32);

        let nonzero_lots = acct.get_num_of_nonzero_lots();

        row.push(raw_acct.name.to_string());
        row.push(balance);
        row.push(raw_acct.ticker.to_string());
        row.push(ocb);
        row.push(cb);
        row.push(acct.list_of_lots.borrow().len().to_string());
        row.push(nonzero_lots.to_string());
        rows.push(row);
    }
    let file_name = PathBuf::from("C3_Acct_Sum_with_orig_and_lk_cost_basis.csv");
    let path = PathBuf::from(&settings.export_path.clone());

    let full_path: PathBuf = [path, file_name].iter().collect();
    let buffer = File::create(full_path).unwrap();
    let mut wtr = csv::Writer::from_writer(buffer);

    for row in rows.iter() {
        wtr.write_record(row).expect("Could not write row to CSV file");
    }
    wtr.flush().expect("Could not flush Writer, though file should exist and be complete");
}

pub fn _4_transaction_mvmt_detail_to_csv(
    settings: &ImportProcessParameters,
    raw_acct_map: &HashMap<u16, RawAccount>,
    acct_map: &HashMap<u16, Account>,
    ars: &HashMap<u32, ActionRecord>,
    txns_map: &HashMap<u32, Transaction>,
) -> Result<(), Box<dyn Error>> {

    let mut rows: Vec<Vec<String>> = [].to_vec();

    let columns = [
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
    ];

    let total_columns = columns.len();

    let mut header: Vec<String> = Vec::with_capacity(total_columns);
    header.extend_from_slice(&columns);
    rows.push(header);

    let length = txns_map.len();

    for txn_num in 1..=length {

        let txn_num = txn_num as u32;
        let txn = txns_map.get(&(txn_num)).unwrap();

        let flow_or_outgoing_exchange_movements = txn.get_outgoing_exchange_and_flow_mvmts(
            &settings.home_currency,
            ars,
            raw_acct_map,
            acct_map,
            txns_map
        )?;

        for mvmt in flow_or_outgoing_exchange_movements.iter() {
            let lot = mvmt.get_lot(acct_map, ars);
            let acct = acct_map.get(&lot.account_key).unwrap();
            let raw_acct = raw_acct_map.get(&acct.raw_key).unwrap();

            let date = txn.date.to_string();
            let tx_number = txn.tx_number.to_string();
            let tx_type = txn.transaction_type(&ars, &raw_acct_map, &acct_map)?;
            let tx_type_string = mvmt.friendly_tx_type(&tx_type);
            let memo = txn.user_memo.to_string();
            let mut amount = d128!(0);
            amount += mvmt.amount;   //  To prevent printing -5E+1 instead of 50, for example
            let ticker = raw_acct.ticker.to_string();
            let term = mvmt.get_term(acct_map, ars).to_string();
            let mut proceeds_lk = mvmt.proceeds_lk.get();
            let mut cost_basis_lk = mvmt.cost_basis_lk.get();
            let mut gain_loss = mvmt.get_lk_gain_or_loss();
            let income = mvmt.get_income(ars, &raw_acct_map, &acct_map, &txns_map)?;
            let expense = mvmt.get_expense(ars, &raw_acct_map, &acct_map, &txns_map)?;


            if tx_type == TxType::Flow && amount > d128!(0) {
                proceeds_lk = d128!(0);
                cost_basis_lk = d128!(0);
                gain_loss = d128!(0);
            }

            let mut row: Vec<String> = Vec::with_capacity(total_columns);

            row.push(date);
            row.push(tx_number.to_string());
            row.push(tx_type_string);
            row.push(memo);
            row.push(amount.to_string());
            row.push(ticker);
            row.push(term);
            row.push(proceeds_lk.to_string());
            row.push(cost_basis_lk.to_string());
            row.push(gain_loss.to_string());
            row.push(income.to_string());
            row.push(expense.to_string());
            rows.push(row);
        }
    }

    let file_name = PathBuf::from("C4_Txns_mvmts_detail.csv");
    let path = PathBuf::from(&settings.export_path);

    let full_path: PathBuf = [path, file_name].iter().collect();
    let buffer = File::create(full_path).unwrap();
    let mut wtr = csv::Writer::from_writer(buffer);

    for row in rows.iter() {
        wtr.write_record(row).expect("Could not write row to CSV file");
    }
    wtr.flush().expect("Could not flush Writer, though file should exist and be complete");

    Ok(())
}

pub fn _5_transaction_mvmt_summaries_to_csv(
    settings: &ImportProcessParameters,
    raw_acct_map: &HashMap<u16, RawAccount>,
    acct_map: &HashMap<u16, Account>,
    ars: &HashMap<u32, ActionRecord>,
    txns_map: &HashMap<u32, Transaction>,
) -> Result<(), Box<dyn Error>> {

    let mut rows: Vec<Vec<String>> = [].to_vec();

    let columns = [
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
    ];

    let total_columns = columns.len();
    let mut header: Vec<String> = Vec::with_capacity(total_columns);

    header.extend_from_slice(&columns);
    rows.push(header);

    let length = txns_map.len();

    let mut tx_type_string = "".to_string();

    for txn_num in 1..=length {

        let txn_num = txn_num as u32;
        let txn = txns_map.get(&(txn_num)).unwrap();
        let txn_date_string = txn.date.to_string();
        let tx_num_string = txn.tx_number.to_string();
        let tx_type = txn.transaction_type(ars, &raw_acct_map, &acct_map)?;
        let tx_memo_string = txn.user_memo.to_string();
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
            &settings.home_currency,
            ars,
            raw_acct_map,
            acct_map,
            txns_map
        )?;

        let mut count = 0;
        for mvmt in flow_or_outgoing_exchange_movements.iter() {
            let lot = mvmt.get_lot(acct_map, ars);
            let acct = acct_map.get(&lot.account_key).unwrap();
            let raw_acct = raw_acct_map.get(&acct.raw_key).unwrap();

            if count == 0 { tx_type_string = mvmt.friendly_tx_type(&tx_type) };
            count += 1;

            if ticker.is_none() { ticker = Some(raw_acct.ticker.clone()) };

            if polarity.is_none() {
                polarity = if mvmt.amount > d128!(0) {
                    Some(Polarity::Incoming)
                    } else { Some(Polarity::Outgoing)
                };
            }

            let term = mvmt.get_term(acct_map, ars);

            if term == Term::LT {
                amount_lt += mvmt.amount;
                proceeds_lt += mvmt.proceeds_lk.get();
                cost_basis_lt += mvmt.cost_basis_lk.get();
                if term_lt.is_none() { term_lt = Some(term) }
            } else {
                assert_eq!(term, Term::ST);
                amount_st += mvmt.amount;
                proceeds_st += mvmt.proceeds_lk.get();
                cost_basis_st += mvmt.cost_basis_lk.get();
                if term_st.is_none() {
                    term_st = Some(term);
                }
            }
        }

        if (txn.transaction_type(
            ars,
            &raw_acct_map,
            &acct_map)? == TxType::Flow
        ) & (polarity == Some(Polarity::Incoming)) {
            income_st = -proceeds_st;   //  Proceeds are negative for incoming txns
            proceeds_st = d128!(0);
            cost_basis_st = d128!(0);
            income_lt = -proceeds_lt;   //  Proceeds are negative for incoming txns
            proceeds_lt = d128!(0);
            cost_basis_lt = d128!(0);
        }

        if (txn.transaction_type(
            ars,
            &raw_acct_map,
            &acct_map)? == TxType::Flow
        ) & (polarity == Some(Polarity::Outgoing)) {
            expense_st -= proceeds_st;
            expense_lt -= proceeds_lt;
        }

        if let Some(term) = term_st {

            let mut row: Vec<String> = Vec::with_capacity(total_columns);

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

            let mut row: Vec<String> = Vec::with_capacity(total_columns);

            row.push(txn_date_string);
            row.push(tx_num_string);
            row.push(tx_type_string.clone());
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

    let file_name = PathBuf::from("C5_Txns_mvmts_summary.csv");
    let path = PathBuf::from(&settings.export_path);

    let full_path: PathBuf = [path, file_name].iter().collect();
    let buffer = File::create(full_path).unwrap();
    let mut wtr = csv::Writer::from_writer(buffer);

    for row in rows.iter() {
        wtr.write_record(row).expect("Could not write row to CSV file");
    }
    wtr.flush().expect("Could not flush Writer, though file should exist and be complete");

    Ok(())
}

pub fn _6_transaction_mvmt_detail_to_csv_w_orig(
    settings: &ImportProcessParameters,
    raw_acct_map: &HashMap<u16, RawAccount>,
    acct_map: &HashMap<u16, Account>,
    ars: &HashMap<u32, ActionRecord>,
    txns_map: &HashMap<u32, Transaction>,
) -> Result<(), Box<dyn Error>> {

    let mut rows: Vec<Vec<String>> = [].to_vec();

    let lk = settings.lk_treatment_enabled;

    let columns = [
        "Date".to_string(),
        "Txn#".to_string(),
        "Type".to_string(),
        "User Memo".to_string(),
        "Auto Memo".to_string(),
        "Amount".to_string(),
        "Ticker".to_string(),
        "Term".to_string(),
        "Proceeds".to_string(),
        "Cost basis".to_string(),
        "Gain/loss".to_string(),
        "Income".to_string(),
        "Expense".to_string(),
    ];

    let lk_columns = [
        "Orig. Proceeds".to_string(),
        "Orig. Cost basis".to_string(),
        "Orig. Gain/loss".to_string(),
    ];

    let total_columns = if lk {
        columns.len() + lk_columns.len()
    } else {
        columns.len()
    };

    let mut header: Vec<String> = Vec::with_capacity(total_columns);

    header.extend_from_slice(&columns);

    if lk {
        header.extend_from_slice(&lk_columns)
    }
    rows.push(header);

    let length = txns_map.len();

    for txn_num in 1..=length {

        let txn_num = txn_num as u32;
        let txn = txns_map.get(&(txn_num)).unwrap();

        let flow_or_outgoing_exchange_movements = txn.get_outgoing_exchange_and_flow_mvmts(
            &settings.home_currency,
            ars,
            raw_acct_map,
            acct_map,
            txns_map
        )?;

        for mvmt in flow_or_outgoing_exchange_movements.iter() {
            let lot = mvmt.get_lot(acct_map, ars);
            let acct = acct_map.get(&lot.account_key).unwrap();
            let raw_acct = raw_acct_map.get(&acct.raw_key).unwrap();

            let date = txn.date.to_string();
            let tx_number = txn.tx_number.to_string();
            let tx_type = txn.transaction_type(&ars, &raw_acct_map, &acct_map)?;
            let tx_type_string = mvmt.friendly_tx_type(&tx_type);
            let user_memo = txn.user_memo.to_string();
            let auto_memo = txn.get_auto_memo(ars, raw_acct_map,acct_map, &settings.home_currency)?;
            let mut amount = d128!(0);
            amount += mvmt.amount;   //  To prevent printing -5E+1 instead of 50, for example
            let ticker = raw_acct.ticker.to_string();
            let term = mvmt.get_term(acct_map, ars).to_string();
            let mut proceeds_lk = mvmt.proceeds_lk.get();
            let mut cost_basis_lk = mvmt.cost_basis_lk.get();
            let mut gain_loss = mvmt.get_lk_gain_or_loss();
            let income = mvmt.get_income(ars, &raw_acct_map, &acct_map, &txns_map)?;
            let expense = mvmt.get_expense(ars, &raw_acct_map, &acct_map, &txns_map)?;
            let mut orig_proc = mvmt.proceeds.get();
            let mut orig_cost = mvmt.cost_basis.get();
            let mut orig_gain_loss = mvmt.get_orig_gain_or_loss();

            if tx_type == TxType::Flow && amount > d128!(0) {
                proceeds_lk = d128!(0);
                cost_basis_lk = d128!(0);
                gain_loss = d128!(0);
                orig_proc = d128!(0);
                orig_cost = d128!(0);
                orig_gain_loss = d128!(0);
            }

            let mut row: Vec<String> = Vec::with_capacity(total_columns);

            row.push(date);
            row.push(tx_number);
            row.push(tx_type_string);
            row.push(user_memo);
            row.push(auto_memo);
            row.push(amount.to_string());
            row.push(ticker);
            row.push(term);
            row.push(proceeds_lk.to_string());
            row.push(cost_basis_lk.to_string());
            row.push(gain_loss.to_string());
            row.push(income.to_string());
            row.push(expense.to_string());
            if lk {
                row.push(orig_proc.to_string());
                row.push(orig_cost.to_string());
                row.push(orig_gain_loss.to_string());
            }
            rows.push(row);
        }
    }

    let file_name = PathBuf::from("C6_Txns_mvmts_more_detail.csv");
    let path = PathBuf::from(&settings.export_path);

    let full_path: PathBuf = [path, file_name].iter().collect();
    let buffer = File::create(full_path).unwrap();
    let mut wtr = csv::Writer::from_writer(buffer);

    for row in rows.iter() {
        wtr.write_record(row).expect("Could not write row to CSV file");
    }
    wtr.flush().expect("Could not flush Writer, though file should exist and be complete");

    Ok(())
}
