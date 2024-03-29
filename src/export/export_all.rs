// Copyright (c) 2017-2023, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools/blob/master/LEGAL.txt

use std::error::Error;
use std::collections::HashMap;

use crptls::transaction::{Transaction, ActionRecord};
use crptls::account::{Account, RawAccount};
use crptls::core_functions::ImportProcessParameters;
use crate::export::{export_csv, export_txt, export_je};


pub fn export(
    settings: &ImportProcessParameters,
    raw_acct_map: &HashMap<u16, RawAccount>,
    account_map: &HashMap<u16, Account>,
    action_records_map: &HashMap<u32, ActionRecord>,
    transactions_map: &HashMap<u32, Transaction>,
) -> Result<(), Box<dyn Error>> {

    println!("Creating all reports now.");

    export_csv::_1_account_sums_to_csv(
        &settings,
        &raw_acct_map,
        &account_map
    );

    export_csv::_2_account_sums_nonzero_to_csv(
        &settings,
        &raw_acct_map,
        &account_map,
    );

    if settings.lk_treatment_enabled {
        export_csv::_3_account_sums_to_csv_with_orig_basis(
            &settings,
            &raw_acct_map,
            &account_map
        );
    }

    export_csv::_4_transaction_mvmt_detail_to_csv(
        &settings,
        &raw_acct_map,
        &account_map,
        &action_records_map,
        &transactions_map
    )?;

    export_csv::_5_transaction_mvmt_summaries_to_csv(
        &settings,
        &raw_acct_map,
        &account_map,
        &action_records_map,
        &transactions_map
    )?;

    export_csv::_6_transaction_mvmt_detail_to_csv_w_orig(
        &settings,
        &raw_acct_map,
        &account_map,
        &action_records_map,
        &transactions_map
    )?;

    export_csv::_7_gain_loss_8949_to_csv(
        &settings,
        &raw_acct_map,
        &account_map,
        &action_records_map,
        &transactions_map
    )?;

    export_txt::_1_account_lot_detail_to_txt(
        &settings,
        &raw_acct_map,
        &account_map,
        &action_records_map,
        &transactions_map,
    )?;

    export_txt::_2_account_lot_summary_to_txt(
        &settings,
        &raw_acct_map,
        &account_map,
    )?;

    export_txt::_3_account_lot_summary_non_zero_to_txt(
        &settings,
        &raw_acct_map,
        &account_map,
    )?;

    if !settings.lk_treatment_enabled {
        export_je::prepare_non_lk_journal_entries(
            &settings,
            &raw_acct_map,
            &account_map,
            &action_records_map,
            &transactions_map,
        )?;
    }

Ok(())
}