// Copyright (c) 2017-2019, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools/blob/master/LEGAL.txt

use std::error::Error;
use std::process;
use std::io::{self, BufRead};
use std::path::PathBuf;
use std::collections::{HashMap};

use crate::cli_user_choices;
use crate::core_functions::{self, LikeKindSettings, ImportProcessParameters};
use crate::account::{Account, RawAccount};
use crate::transaction::{Transaction, ActionRecord};

pub(crate) fn wizard(args: super::Cli) -> Result<(
    HashMap<u16, Account>,
    HashMap<u16, RawAccount>,
    HashMap<u32, ActionRecord>,
    HashMap<u32, Transaction>,
    Option<LikeKindSettings>,
    ImportProcessParameters,
    bool
), Box<dyn Error>> {

    shall_we_proceed()?;

    let date_separator = match args.date_separator.into_string().unwrap().as_str() {
        "h" => {"-"}
        "s" => {"/"}
        _ => { println!("\nFATAL: The date-separator arg requires either an 'h' or an 's'.\n"); process::exit(1) }
    };

    let input_file_path;

    if let Some(file) = args.file_to_import {
        input_file_path = file
    } else {
        input_file_path = cli_user_choices::choose_file_for_import()?;
    }

    let output_dir_path = args.output_dir_path;

    let costing_method_choice = cli_user_choices::choose_inventory_costing_method(args.inv_costing_method)?;

    let home_currency_choice = args.home_currency.into_string().unwrap().to_uppercase();

    let lk_cutoff_date_opt_string;

    if let Some(lk_cutoff) = args.lk_cutoff_date {
        lk_cutoff_date_opt_string = Some(lk_cutoff.into_string().unwrap())
    } else {
        lk_cutoff_date_opt_string = None
    };

    let (like_kind_election, like_kind_cutoff_date) = cli_user_choices::elect_like_kind_treatment(&lk_cutoff_date_opt_string)?;

    let mut settings = ImportProcessParameters {
        export_path: output_dir_path,
        home_currency: home_currency_choice,
        costing_method: costing_method_choice,
        enable_like_kind_treatment: like_kind_election,
        lk_cutoff_date_string: like_kind_cutoff_date,
        date_separator: date_separator.to_string(),
    };

    let (
        account_map1,
        raw_acct_map1,
        action_records_map1,
        transactions_map1,
        like_kind_settings1
    ) = core_functions::import_and_process_final(input_file_path, &settings)?;

    let should_export = export_reports_to_output_dir(&mut settings)?;

    Ok((account_map1, raw_acct_map1, action_records_map1, transactions_map1, like_kind_settings1, settings, should_export))
}

fn shall_we_proceed() -> Result<(), Box<dyn Error>> {

    println!("Shall we proceed? [Y/n] ");

    _proceed()?;

    fn _proceed() -> Result<(), Box<dyn Error>> {

        let mut input = String::new();
        let stdin = io::stdin();
        stdin.lock().read_line(&mut input)?;

        match input.trim().to_ascii_lowercase().as_str() {

            "y" | "ye" | "yes" | "" => { Ok(()) },
            "n" | "no" => { println!("We have NOT proceeded..."); process::exit(0); },
            _   => { println!("Please respond with 'y' or 'n' (or 'yes' or 'no')."); _proceed() }
        }
    }

    Ok(())
}

fn export_reports_to_output_dir(settings: &mut ImportProcessParameters) -> Result<(bool), Box<dyn Error>> {

    println!("\nThe directory currently selected for exporting reports is: {}", settings.export_path.to_str().unwrap());

    if settings.export_path.to_str().unwrap() == "." {
        println!("  (A 'dot' denotes the default value: current working directory.)");
    }

    println!("\nExport reports to selected directory? [Y/n/c] ('c' to 'change') ");

    let choice = _export(settings)?;

    fn _export(settings: &mut ImportProcessParameters) -> Result<(bool), Box<dyn Error>> {

        let mut input = String::new();
        let stdin = io::stdin();
        stdin.lock().read_line(&mut input)?;

        match input.trim().to_ascii_lowercase().as_str() {

            "y" | "ye" | "yes" | "" => { Ok(true) },
            "n" | "no" => { println!("Okay, no reports will be created."); Ok(false) },
            "c" | "change" => {
                let new_dir = cli_user_choices::choose_export_dir()?;
                settings.export_path = PathBuf::from(new_dir);
                Ok(true)
            },
            _   => { println!("Please respond with 'y', 'n', or 'c' (or 'yes' or 'no' or 'change').");
                _export(settings)
            }
        }
    }

    Ok(choice)
}