// Copyright (c) 2017-2019, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools-rs/LEGAL.txt

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
// Note: the above are possibly temporary, to silence "x was not used" warnings.
// #[warn(dead_code)] is the default (same for unused_variables)


use std::ffi::OsString;
use structopt::StructOpt;
use std::path::PathBuf;
use std::process;
use std::io::{self, BufRead};
use std::error::Error;

mod account;
mod transaction;
mod core_functions;
mod import_accts_txns;
mod create_lots_mvmts;
mod import_cost_proceeds_etc;
mod user_choices;
mod export;
mod utils;
mod tests;

use crate::user_choices::LotProcessingChoices;


#[derive(StructOpt, Debug)]
#[structopt(name = "cryptools-rs")]
struct Cli {

    /// File to be imported.  (Currently, the only supported date format is %m/%d/%y.)
    #[structopt(name = "file", parse(from_os_str))]
    file_to_import: Option<PathBuf>,

    /// Output directory for exported reports.
    #[structopt(name = "output directory", short, long = "output", default_value = ".", parse(from_os_str))]
    output_dir_path: PathBuf,

    /// This will prevent the program from writing the CSV to file. This will be ignored if -a is not set (the wizard will always ask to output).
    #[structopt(name = "suppress reports", short, long = "suppress")]
    suppress_reports: bool,

    /// Cutoff date through which like-kind exchange treatment should be applied.
    /// Please use %y-%m-%d (or %Y-%m-%d) format for like-kind cutoff date entry.
    #[structopt(name = "like-kind cutoff date", short, long = "cutoff", parse(from_os_str))]
    cutoff_date: Option<OsString>,

    /// Inventory costing method (in terms of lot selection, i.e., LIFO, FIFO, etc.). There are currently four options (1 through 4).
    #[structopt(name = "method", short, long, default_value = "1", parse(from_os_str), long_help =
    r"    1. LIFO according to the order the lot was created.
    2. LIFO according to the basis date of the lot.
    3. FIFO according to the order the lot was created.
    4. FIFO according to the basis date of the lot.
    ")]
    inv_costing_method: OsString,

    /// Home currency (currency in which all resulting reports are denominated). (Only available as a command line setting.)
    #[structopt(name = "home currency", short = "c", long = "currency", default_value = "USD", parse(from_os_str))]
    home_currency: OsString,

    /// User is instructing the program to skip the data entry wizard. When set, program will error without required command-line args.
    #[structopt(name = "accept args", short, long = "accept")]
    accept_args: bool,
}


fn main() -> Result<(), Box<dyn std::error::Error>> {

    let args = Cli::from_args();

    println!(
    "
    Hello, crypto-folk!  Welcome to cryptools-rs!

    This software will import your csv file's ledger of cryptocurrency transactions.
    It will then process it by creating 'lots' and posting 'movements' to those lots.
    Along the way, it will keep track of income, expenses, gains, and losses.

    Note: it is designed to import a full history. Gains and losses may be incorrect otherwise.
    ");

    let input_file_path;
    let output_dir_path = args.output_dir_path;
    let should_export;

    let account_map;
    let raw_acct_map;
    let action_records_map;
    let transactions_map;
    let mut settings;
    let like_kind_settings;

    let home_currency_choice = args.home_currency.into_string().expect("Home currency should be in the form of a ticker in CAPS.");
    let costing_method_choice;


    if !args.accept_args {

        shall_we_proceed();

        fn shall_we_proceed() {

            println!("Shall we proceed? [Y/n] ");

            match _proceed() {
                Ok(()) => {}
                Err(err) => { println!("Failure to proceed.  {}", err); process::exit(1); }
            };

            fn _proceed() -> Result<(), Box<Error>> {

                let mut input = String::new();
                let stdin = io::stdin();
                stdin.lock().read_line(&mut input)?;

                match input.trim().to_ascii_lowercase().as_str() {
                    "y" | "ye" | "yes" | "" => { Ok(()) },
                    "n" | "no" => { println!("We have NOT proceeded..."); process::exit(0); },
                    _   => { println!("Please respond with 'y' or 'n' (or 'yes' or 'no')."); _proceed() }
                }
            }
        }

        if let Some(file) = args.file_to_import {
            input_file_path = file
        } else {
            input_file_path = user_choices::choose_file_for_import();
        }

        costing_method_choice = LotProcessingChoices::choose_inventory_costing_method();

        let lk_cutoff_date_opt_string;

        if let Some(lk_cutoff) = args.cutoff_date {
            lk_cutoff_date_opt_string = Some(lk_cutoff.into_string().unwrap())
        } else {
            lk_cutoff_date_opt_string = None
        };

        let (like_kind_election, like_kind_cutoff_date) = LotProcessingChoices::elect_like_kind_treatment(&lk_cutoff_date_opt_string);

        settings = LotProcessingChoices {
            export_path: output_dir_path,
            home_currency: home_currency_choice,
            costing_method: costing_method_choice,
            enable_like_kind_treatment: like_kind_election,
            lk_cutoff_date_string: like_kind_cutoff_date,
        };

        let (
            account_map1,
            raw_acct_map1,
            action_records_map1,
            transactions_map1,
            like_kind_settings1
        ) = core_functions::import_and_process_final(input_file_path, &settings);

        account_map = account_map1;
        raw_acct_map = raw_acct_map1;
        action_records_map = action_records_map1;
        transactions_map = transactions_map1;
        like_kind_settings = like_kind_settings1;

        should_export = export_reports_to_output_dir(&mut settings);

        fn export_reports_to_output_dir(settings: &mut LotProcessingChoices) -> bool {

            println!("\nThe directory currently selected for exporting reports is: {}", settings.export_path.to_str().unwrap());

            if &settings.export_path.to_str().unwrap() == &"." {
                println!("  (A 'dot' denotes the default value: current working directory.)");
            }
            println!("\nExport reports to selected directory? [Y/n/c] ('c' to 'change') ");

            let choice = match _export(settings) {
                Ok(choice) => { choice }
                Err(err) => { println!("Export choice error.  {}", err); process::exit(1); }
            };

            fn _export(settings: &mut LotProcessingChoices) -> Result<(bool), Box<Error>> {

                let mut input = String::new();
                let stdin = io::stdin();
                stdin.lock().read_line(&mut input)?;

                match input.trim().to_ascii_lowercase().as_str() {

                    "y" | "ye" | "yes" | "" => { println!("Creating reports now."); Ok(true) },
                    "n" | "no" => { println!("Okay, no reports were created."); Ok(false) },
                    "c" | "change" => {
                        let new_dir = user_choices::choose_export_dir();
                        settings.export_path = PathBuf::from(new_dir);
                        println!("Creating reports now in newly chosen path.");
                        Ok(true)
                    },
                    _   => { println!("Please respond with 'y', 'n', or 'c' (or 'yes' or 'no' or 'change').");
                        _export(settings)
                    }
                }
            }

            choice
        }

    } else {

        if let Some(file) = args.file_to_import {
            input_file_path = file
        } else {
            println!("Flag to 'accept args' was set, but 'file' is missing, though it is a required field. Exiting.");
            process::exit(0);
        }

        let like_kind_election;
        let like_kind_cutoff_date_string: String;

        if let Some(date) = args.cutoff_date {
            like_kind_election = true;
            like_kind_cutoff_date_string = date.into_string().unwrap();
        } else {
            like_kind_election = false;
            like_kind_cutoff_date_string = "1-1-1".to_string();
        };

        match args.inv_costing_method.clone().into_string().expect("Invalid choice on costing method. Aborting.").trim() {
            "1" | "2" | "3" | "4" => {}
            _ => { println!("Invalid choice for inventory costing method. Exiting."); process::exit(0); }
        }

        let costing_method_choice = LotProcessingChoices::inv_costing_from_cmd_arg(args.inv_costing_method.into_string().unwrap());

        settings = LotProcessingChoices {
            export_path: output_dir_path,
            home_currency: home_currency_choice,
            costing_method: costing_method_choice,
            enable_like_kind_treatment: like_kind_election,
            lk_cutoff_date_string: like_kind_cutoff_date_string,
        };

        let (
            account_map1,
            raw_acct_map1,
            action_records_map1,
            transactions_map1,
            like_kind_settings1
        ) = core_functions::import_and_process_final(input_file_path, &settings);

        account_map = account_map1;
        raw_acct_map = raw_acct_map1;
        action_records_map = action_records_map1;
        transactions_map = transactions_map1;
        like_kind_settings = like_kind_settings1;

        should_export = !args.suppress_reports;
    }

    if should_export {

        export::_1_account_sums_to_csv(
            &settings,
            &raw_acct_map,
            &account_map
        );

        export::_2_account_sums_nonzero_to_csv(
            &account_map,
            &settings,
            &raw_acct_map
        );

        export::_5_transaction_mvmt_summaries_to_csv(
            &settings,
            &action_records_map,
            &raw_acct_map,
            &account_map,
            &transactions_map
        );

    }

    // use tests::test;
    // test::run_tests(
    //     &transactions_map,
    //     &action_records_map,
    //     &account_map
    // );


    Ok(())
    // // export::transactions_to_csv(&transactions);
    // // println!("\nReturned from `fn transactions_to_csv`.  It worked!!  Right?");

    // export::accounts_to_csv(&accounts);
    // println!("\nReturned from `fn accounts_to_csv`.  It worked!!  Right?");
}
