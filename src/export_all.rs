// Copyright (c) 2017-2019, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools/blob/master/LEGAL.txt

use std::error::Error;
use std::collections::{HashMap};

use crate::transaction::{Transaction, ActionRecord};
use crate::account::{Account, RawAccount};
use crate::core_functions::{ImportProcessParameters};
use crate::csv_export;
use crate::txt_export;


pub fn export(
    settings: &ImportProcessParameters,
    action_records_map: &HashMap<u32, ActionRecord>,
    raw_acct_map: &HashMap<u16, RawAccount>,
    account_map: &HashMap<u16, Account>,
    transactions_map: &HashMap<u32, Transaction>,
) -> Result<(), Box<dyn Error>> {

    println!("Creating all reports now.");

    csv_export::_1_account_sums_to_csv(
        &settings,
        &raw_acct_map,
        &account_map
    );

    csv_export::_2_account_sums_nonzero_to_csv(
        &account_map,
        &settings,
        &raw_acct_map
    );

    csv_export::_3_account_sums_to_csv_with_orig_basis(
        &settings,
        &raw_acct_map,
        &account_map
    );

    csv_export::_4_transaction_mvmt_detail_to_csv(
        &settings,
        &action_records_map,
        &raw_acct_map,
        &account_map,
        &transactions_map
    )?;

    csv_export::_5_transaction_mvmt_summaries_to_csv(
        &settings,
        &action_records_map,
        &raw_acct_map,
        &account_map,
        &transactions_map
    )?;

    csv_export::_6_transaction_mvmt_detail_to_csv_w_orig(
        &settings,
        &action_records_map,
        &raw_acct_map,
        &account_map,
        &transactions_map
    )?;

    txt_export::_1_account_lot_detail_to_txt(
        &settings,
        &raw_acct_map,
        &account_map,
        &transactions_map,
        &action_records_map
    )?;

    txt_export::_2_account_lot_summary_to_txt(
        &settings,
        &raw_acct_map,
        &account_map,
    )?;

    txt_export::_3_account_lot_summary_non_zero_to_txt(
        &settings,
        &raw_acct_map,
        &account_map,
    )?;

Ok(())
}