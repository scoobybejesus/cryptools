// Copyright (c) 2017-2019, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools/blob/master/LEGAL.txt

use std::error::Error;
use std::collections::{HashMap};
use std::process;

use crate::cli_user_choices;
use crate::core_functions::{self, LikeKindSettings, ImportProcessParameters};
use crate::account::{Account, RawAccount};
use crate::transaction::{Transaction, ActionRecord};

pub(crate) fn skip_wizard(args: super::Cli) -> Result<(
    HashMap<u16, Account>,
    HashMap<u16, RawAccount>,
    HashMap<u32, ActionRecord>,
    HashMap<u32, Transaction>,
    Option<LikeKindSettings>,
    ImportProcessParameters,
    bool
), Box<dyn Error>> {

    let date_separator = match args.opts.date_separator.into_string().unwrap().as_str() {
        "h" => {"-"}
        "s" => {"/"}
        _ => { println!("\nFATAL: The date-separator arg requires either an 'h' or an 's'.\n"); process::exit(1) }
    };

    let input_file_path;

    if let Some(file) = args.file_to_import {
        input_file_path = file
    } else {
        println!("WARN: Flag to 'accept args' was set, but 'file' is missing. It is a required field.\n");
        input_file_path = cli_user_choices::choose_file_for_import()?;
    }

    let output_dir_path = args.opts.output_dir_path;

    let costing_method_choice = cli_user_choices::inv_costing_from_cmd_arg(args.opts.inv_costing_method)?;

    let home_currency_choice = args.opts.home_currency.into_string().unwrap().to_uppercase();

    let like_kind_election;
    let like_kind_cutoff_date_string: String;

    if let Some(date) = args.opts.lk_cutoff_date {
        like_kind_election = true;
        like_kind_cutoff_date_string = date.into_string().unwrap();
    } else {
        like_kind_election = false;
        like_kind_cutoff_date_string = "1-1-1".to_string();
    };

    let settings = ImportProcessParameters {
        export_path: output_dir_path,
        home_currency: home_currency_choice,
        costing_method: costing_method_choice,
        enable_like_kind_treatment: like_kind_election,
        lk_cutoff_date_string: like_kind_cutoff_date_string,
        date_separator: date_separator.to_string(),
        iso_date_style: args.flags.iso_date
    };

    let (
        account_map1,
        raw_acct_map1,
        action_records_map1,
        transactions_map1,
        like_kind_settings1
    ) = core_functions::import_and_process_final(input_file_path, &settings)?;

    let should_export = !args.flags.suppress_reports;

    Ok((account_map1, raw_acct_map1, action_records_map1, transactions_map1, like_kind_settings1, settings, should_export))
}