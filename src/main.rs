// Copyright (c) 2017-2020, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools/blob/master/LEGAL.txt

// #![allow(dead_code)]
// #![allow(unused_variables)]
// #![allow(unused_assignments)]
// Note: the above are possibly temporary, to silence "x was not used" warnings.
// #[warn(dead_code)] is the default (same for unused_variables)


use std::path::PathBuf;
use std::error::Error;

use structopt::StructOpt;

mod setup;
mod cli_user_choices;
mod wizard;
mod skip_wizard;
mod export;

#[cfg(feature = "print_menu")]
mod mytui;

use export::{export_all, export_je};


#[derive(StructOpt, Debug)]
#[structopt(name = "cryptools")]
pub struct Cli {

    /// User is instructing the program to skip the data entry wizard.
    /// When set, default settings will be assumed if they are not set by 
    /// environment variables (or .env file) or certain command line flags.
    #[structopt(name = "accept args", short = "a", long = "accept")]
    accept_args: bool,

    /// Suppresses the printing of "all" reports, except that it *will* trigger the
    /// exporting of a txt file containing an accounting journal entry for every transaction.
    /// Individual account and transaction reports may still be printed via the print_menu
    /// with the -p flag. Note: the journal entries are not suitable for like-kind transactions.
    #[structopt(name = "journal entries", short, long = "journal-entries")]
    journal_entries_only: bool,

    /// Once the file_to_import has been fully processed, the user will be presented
    /// with a menu for manually selecting which reports to print/export. If this flag is not
    /// set, the program will print/export all available reports.
    #[cfg(feature = "print_menu")]
    #[structopt(name = "print menu", short, long = "print-menu")]
    print_menu: bool,

    /// Prevents the program from writing reports to files.
    /// This will be ignored if -a is not set (the wizard will always ask to output).
    #[structopt(name = "suppress reports", short, long = "suppress")]
    suppress_reports: bool,

    /// Output directory for exported reports.
    #[structopt(name = "output directory", short, long = "output", default_value = ".", parse(from_os_str))]
    output_dir_path: PathBuf,

    /// Causes the program to expect the `txDate` field in the file_to_import to use the format YYYY-MM-dd
    /// or YY-MM-dd (or YYYY/MM/dd or YY/MM/dd) instead of the default US-style MM-dd-YYYY or MM-dd-YY 
    /// (or MM/dd/YYYY or MM/dd/YY).
    /// NOTE: this flag overrides the ISO_DATE environment variable, including if set in the .env file.
    #[structopt(name = "imported file uses ISO 8601 date format", short = "i", long = "iso")]
    iso_date: bool,

    /// Tells the program a non-default date separator (instead of a hyphen "-", a slash "/") was used
    /// in the file_to_import `txDate` column (i.e. 2017-12-31 instead of 2017/12/31).
    /// NOTE: this flag overrides the DATE_SEPARATOR_IS_SLASH environment variable, including if set in the .env file.
    #[structopt(name = "date separator character is slash", short = "d", long = "date-separator-is-slash")]
    date_separator_is_slash: bool,

    /// File to be imported.  Some notes on the columns: (a) by default, the program expects the `txDate` column to 
    /// be formatted as %m-%d-%y. You may alter this with ISO_DATE and DATE_SEPARATOR_IS_SLASH flags or environment
    /// variables; (b) the `proceeds` column and any values in transactions must have a period (".") as the decimal
    /// separator; and (c) any transactions with negative values must not be wrapped in parentheses (use the python
    /// script for sanitizing/converting negative values).
    /// See .env.example for further details on environment variables.
    #[structopt(name = "file_to_import", parse(from_os_str))]
    file_to_import: Option<PathBuf>,
}

/// These are the values able to be captured from environment variables.
#[derive(Debug)]
pub struct Cfg {
    /// Setting the corresponding environment variable to `true` (or `1`) will cause the program to expect the `txDate` field in the 
    /// `Cli::file_to_import` to use the format YYYY-MM-dd or YY-MM-dd (or YYYY/MM/dd or YY/MM/dd, depending on the date-separator option).
    /// The default value is `false`, meaning the program will expect default US-style MM-dd-YYYY or MM-dd-YY (or MM/dd/YYYY 
    /// or MM/dd/YY, depending on the date separator option).   
    iso_date: bool,
    /// Switches the default date separator from hyphen to slash (i.e., from "-" to "/") to indicate the separator
    /// character used in the file_to_import txDate column (i.e. 2017-12-31 to 2017/12/31).
    date_separator_is_slash: bool,
    /// Home currency (currency from the `proceeds` column of the `Cli::file_to_import` and in which all resulting reports are denominated).  
    /// Default is `USD`.
    home_currency: String,
    /// Cutoff date through which like-kind exchange treatment should be applied. You must use %y-%m-%d (or %Y-%m-%d)
    /// format for like-kind cutoff date entry.  The default is blank/commented/`None`.
    lk_cutoff_date: Option<String>,
    /// method number for lot selection <method number for lot selection>
    /// 1. LIFO according to the order the lot was created.
    /// 2. LIFO according to the basis date of the lot.
    /// 3. FIFO according to the order the lot was created.
    /// 4. FIFO according to the basis date of the lot.
     /// [default: 1]
    inv_costing_method: String,
}

fn main() -> Result<(), Box<dyn Error>> {

    let args = Cli::from_args();

    println!(
        "\
Hello!

This software will import your csv file's ledger of cryptocurrency transactions.
It will then process it by creating 'lots' and posting 'movements' to those lots.
Along the way, it will keep track of income, expenses, gains, and losses.

See examples/.env.example or run with --help to learn how to change default program behavior.

  Note: The software is designed to import a full history. Gains and losses may be incorrect otherwise.
    ");

    let cfg = setup::get_env(&args)?;

    let (input_file_path, settings) = setup::run_setup(&args, cfg)?;

    let (
        raw_acct_map,
        account_map,
        action_records_map,
        transactions_map,
    ) = crptls::core_functions::import_and_process_final(input_file_path, &settings)?;

    let mut should_export_all = settings.should_export;

    #[cfg(feature = "print_menu")]
    let present_print_menu_tui: bool = args.print_menu.to_owned();

    #[cfg(feature = "print_menu")]
    if present_print_menu_tui { should_export_all = false }

    let print_journal_entries_only = settings.journal_entry_export;
    if print_journal_entries_only { should_export_all = false }

    if should_export_all {

        export_all::export(
            &settings,
            &raw_acct_map,
            &account_map,
            &action_records_map,
            &transactions_map
        )?;
    }

    if print_journal_entries_only && !settings.lk_treatment_enabled {

        export_je::prepare_non_lk_journal_entries(
            &settings,
            &raw_acct_map,
            &account_map,
            &action_records_map,
            &transactions_map,
        )?;
    }

    #[cfg(feature = "print_menu")]
    if present_print_menu_tui {

        mytui::print_menu_tui::print_menu_tui(
            &settings,
            &raw_acct_map,
            &account_map,
            &action_records_map,
            &transactions_map
        )?;
    }

    // use tests::test;
    // test::run_tests(
    //     &transactions_map,
    //     &action_records_map,
    //     &account_map
    // );


    Ok(())

}
