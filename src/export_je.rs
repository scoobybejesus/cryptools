// Copyright (c) 2017-2019, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools/blob/master/LEGAL.txt

use std::fs::{OpenOptions};
use std::collections::{HashMap};
use std::path::PathBuf;
use std::error::Error;
use std::io::prelude::Write;

use decimal::d128;

use crptls::transaction::{Transaction, ActionRecord, Polarity, TxType};
use crptls::account::{Account, RawAccount, Term};
use crptls::core_functions::{ImportProcessParameters};


pub fn prepare_non_lk_journal_entries(
    settings: &ImportProcessParameters,
    ars: &HashMap<u32, ActionRecord>,
    raw_acct_map: &HashMap<u16, RawAccount>,
    acct_map: &HashMap<u16, Account>,
    txns_map: &HashMap<u32, Transaction>,
)  -> Result<(), Box<dyn Error>> {

    let file_name = PathBuf::from("J1_Journal_Entries.txt");
    let path = PathBuf::from(&settings.export_path.clone());
    let full_path: PathBuf = [path, file_name].iter().collect();

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(full_path)?;

    writeln!(file, "Journal Entries
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

    let length = txns_map.len();

    for txn_num in 1..=length {

        let txn_num = txn_num as u32;
        let txn = txns_map.get(&(txn_num)).unwrap();
        let date = txn.date;
        let user_memo = txn.user_memo.to_string();
        let auto_memo = txn.get_auto_memo(ars, raw_acct_map,acct_map, &settings.home_currency)?;
        let tx_type = txn.transaction_type(&ars, &raw_acct_map, &acct_map)?;

        writeln!(file, "\n=====================================\n")?;
        writeln!(file, "Txn {} on {}. {}. {}",
            txn_num,
            date,
            user_memo,
            auto_memo,
        )?;

        let mut cost_basis_ic: Option<d128> = None;
        let mut cost_basis_og: Option<d128> = None;

        let mut acct_string_ic = "".to_string();
        let mut acct_string_og = "".to_string();

        for ar_num in txn.action_record_idx_vec.iter() {

            let ar = ars.get(ar_num).unwrap();
            let acct = acct_map.get(&ar.account_key).unwrap();
            let raw_acct = raw_acct_map.get(&acct.raw_key).unwrap();

            if ar.direction() == Polarity::Incoming {
                cost_basis_ic = Some(ar.cost_basis_in_ar());
                acct_string_ic = format!("{} - {} ({}) (#{})",
                    raw_acct.name,
                    raw_acct.ticker,
                    raw_acct.margin_string(),
                    raw_acct.account_num,
                );
            } else {
                cost_basis_og = Some(ar.cost_basis_in_ar());
                acct_string_og = format!("{} - {} ({}) (#{})",
                    raw_acct.name,
                    raw_acct.ticker,
                    raw_acct.margin_string(),
                    raw_acct.account_num,
                );
            }
        }

        let mut term_st: Option<Term> = None;
        let mut term_lt: Option<Term> = None;

        let mut polarity: Option<Polarity> = None;

        let mut amount_st = d128!(0);
        let mut proceeds_st = d128!(0);
        let mut cost_basis_st = d128!(0);

        let mut amount_lt = d128!(0);
        let mut proceeds_lt = d128!(0);
        let mut cost_basis_lt = d128!(0);

        let mut income = d128!(0);
        let mut expense = d128!(0);

        let flow_or_outgoing_exchange_movements = txn.get_outgoing_exchange_and_flow_mvmts(
            &settings.home_currency,
            ars,
            raw_acct_map,
            acct_map,
            txns_map
        )?;

        for mvmt in flow_or_outgoing_exchange_movements.iter() {

            if let None = polarity {
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
                match term_lt {
                    None => { term_lt = Some(term)}
                    _ => {}
                }
            } else {
                assert_eq!(term, Term::ST);
                amount_st += mvmt.amount;
                proceeds_st += mvmt.proceeds_lk.get();
                cost_basis_st += mvmt.cost_basis_lk.get();
                if term_st == None {
                    term_st = Some(term);
                }
            }
            income += mvmt.get_income(ars, &raw_acct_map, &acct_map, &txns_map)?;
            expense += mvmt.get_expense(ars, &raw_acct_map, &acct_map, &txns_map)?;
        }

        if (txn.transaction_type(
            ars,
            &raw_acct_map,
            &acct_map)? == TxType::Flow
        ) & (polarity == Some(Polarity::Incoming)) {

            proceeds_st = d128!(0);
            cost_basis_st = d128!(0);

            proceeds_lt = d128!(0);
            cost_basis_lt = d128!(0);
        }

        let lt_gain_loss = proceeds_lt + cost_basis_lt;
        let st_gain_loss = proceeds_st + cost_basis_st;

        let mut debits = d128!(0);
        let mut credits = d128!(0);

        if let Some(cb) = cost_basis_ic {
            debits += cb;
            writeln!(file, "{:50}{:5}{:>20}{:5}{:>20}",
            acct_string_ic,
            "",
            cb.to_string(),
            "",
            "",
            )?;
        }

        if let Some(cb) = cost_basis_og {
            credits += cb;
            writeln!(file, "{:50}{:5}{:>20}{:5}{:>20}",
            acct_string_og,
            "",
            "",
            "",
            cb.to_string(),
            )?;
        }

        if lt_gain_loss != d128!(0) {

            if lt_gain_loss > d128!(0) {
                credits += lt_gain_loss.abs();
                let ltg_string = format!("Long-term gain disposing {}", amount_lt.abs());
                writeln!(file, "{:50}{:5}{:>20}{:5}{:>20}",
                ltg_string,
                "",
                "",
                "",
                lt_gain_loss.to_string(),
                )?;
            } else {
                debits += lt_gain_loss.abs();
                let ltl_string = format!("Long-term loss disposing {}", amount_lt.abs());
                writeln!(file, "{:50}{:5}{:>20}{:5}{:>20}",
                ltl_string,
                "",
                lt_gain_loss.abs().to_string(),
                "",
                "",
                )?;
            }
        }

        if st_gain_loss != d128!(0) {

            if st_gain_loss > d128!(0) {
                credits += st_gain_loss.abs();
                let stg_string = format!("Short-term gain disposing {}", amount_st.abs());
                writeln!(file, "{:50}{:5}{:>20}{:5}{:>20}",
                stg_string,
                "",
                "",
                "",
                st_gain_loss.to_string(),
                )?;
            } else {
                debits += st_gain_loss.abs();
                let stl_string = format!("Short-term loss disposing {}", amount_st.abs());
                writeln!(file, "{:50}{:5}{:>20}{:5}{:>20}",
                stl_string,
                "",
                st_gain_loss.abs().to_string(),
                "",
                "",
                )?;
            }
        }

        if income != d128!(0) {
            credits += income;
            writeln!(file, "{:50}{:5}{:>20}{:5}{:>20}",
            "Income",
            "",
            "",
            "",
            income.to_string(),
            )?;
        }

        if expense != d128!(0) {
            debits += expense.abs();
            writeln!(file, "{:50}{:5}{:>20}{:5}{:>20}",
            "Expense",
            "",
            expense.abs().to_string(),
            "",
            "",
            )?;
        }

        writeln!(file, "{:50}{:5}{:>20}{:5}{:>20}",
            "",
            "",
            "--------------------",
            "",
            "--------------------",
        )?;

        writeln!(file, "{:50}{:5}{:>20}{:5}{:>20}",
            "    Totals",
            "",
            debits,
            "",
            credits,
        )?;

        // if (debits - credits) != d128!(0) {
        //     println!("Rounding issue on transaction #{}", txn_num);
        // }

    }

    Ok(())
}