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

pub(crate) fn wizard(
    non_excl_args: super::NonExclusiveToImportWizardArgs,
    excl_args: super::ExclusiveToImportWizardArgs
) -> Result<(
    HashMap<u16, Account>,
    HashMap<u16, RawAccount>,
    HashMap<u32, ActionRecord>,
    HashMap<u32, Transaction>,
    Option<LikeKindSettings>,
    ImportProcessParameters,
    bool
), Box<dyn Error>> {

    shall_we_proceed()?;

    let costing_method_choice = cli_user_choices::choose_inventory_costing_method(excl_args.inv_costing_method_arg)?;

    let lk_cutoff_date_opt_string;

    if let Some(lk_cutoff) = excl_args.lk_cutoff_date_arg {
        lk_cutoff_date_opt_string = Some(lk_cutoff.into_string().unwrap())
    } else {
        lk_cutoff_date_opt_string = None
    };

    let (like_kind_election, like_kind_cutoff_date_string) = cli_user_choices::elect_like_kind_treatment(&lk_cutoff_date_opt_string)?;

    let mut settings = ImportProcessParameters {
        export_path: excl_args.output_dir_path,
        home_currency: non_excl_args.home_currency,
        costing_method: costing_method_choice,
        enable_like_kind_treatment: like_kind_election,
        lk_cutoff_date_string: like_kind_cutoff_date_string,
        date_separator: non_excl_args.date_separator,
        iso_date_style: non_excl_args.iso_date
    };

    let (
        account_map1,
        raw_acct_map1,
        action_records_map1,
        transactions_map1,
        like_kind_settings1
    ) = core_functions::import_and_process_final(non_excl_args.file_to_import, &settings)?;

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