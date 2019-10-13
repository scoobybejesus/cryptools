// Copyright (c) 2017-2019, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools/blob/master/LEGAL.txt

use std::error::Error;
use std::process;
use std::io::{self, BufRead};
use std::path::PathBuf;

use crate::cli_user_choices;
use crate::core_functions::{InventoryCostingMethod};
use crate::setup::{ArgsForImportVarsTBD};

pub(crate) fn wizard(args: ArgsForImportVarsTBD) -> Result<(
    InventoryCostingMethod,
    bool,
    String,
    bool,
    PathBuf,
), Box<dyn Error>> {

    shall_we_proceed()?;

    let costing_method_choice = cli_user_choices::choose_inventory_costing_method(args.inv_costing_method_arg)?;

    let mut lk_cutoff_date_opt_string;

    if let Some(lk_cutoff) = args.lk_cutoff_date_arg {
        lk_cutoff_date_opt_string = Some(lk_cutoff.into_string().unwrap())
    } else {
        lk_cutoff_date_opt_string = None
    };

    let (like_kind_election, like_kind_cutoff_date_string) = cli_user_choices::elect_like_kind_treatment(&mut lk_cutoff_date_opt_string)?;

    let (should_export, output_dir_path) = export_reports_to_output_dir(args.output_dir_path)?;

    Ok((costing_method_choice, like_kind_election, like_kind_cutoff_date_string, should_export, output_dir_path.to_path_buf()))
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

fn export_reports_to_output_dir(output_dir_path: PathBuf) -> Result<(bool, PathBuf), Box<dyn Error>> {

    println!("\nThe directory currently selected for exporting reports is: {}", output_dir_path.to_str().unwrap());

    if output_dir_path.to_str().unwrap() == "." {
        println!("  (A 'dot' denotes the default value: current working directory.)");
    }

    println!("\nExport reports to selected directory? [Y/n/c] ('c' to 'change') ");

    let (choice, path) = _export(output_dir_path)?;

    fn _export(output_dir_path: PathBuf) -> Result<(bool, PathBuf), Box<dyn Error>> {

        let mut input = String::new();
        let stdin = io::stdin();
        stdin.lock().read_line(&mut input)?;

        match input.trim().to_ascii_lowercase().as_str() {

            "y" | "ye" | "yes" | "" => { Ok((true, output_dir_path)) },
            "n" | "no" => { println!("Okay, no reports will be created."); Ok((false, output_dir_path)) },
            "c" | "change" => {
                let new_dir = cli_user_choices::choose_export_dir()?;
                Ok((true, new_dir))
            },
            _   => { println!("Please respond with 'y', 'n', or 'c' (or 'yes' or 'no' or 'change').");
                _export(output_dir_path)
            }
        }
    }

    Ok((choice, path))
}