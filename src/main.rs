// Copyright (c) 2017-2019, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools/blob/master/LEGAL.txt

// #![allow(dead_code)]
// #![allow(unused_variables)]
// #![allow(unused_assignments)]
// Note: the above are possibly temporary, to silence "x was not used" warnings.
// #[warn(dead_code)] is the default (same for unused_variables)


use std::ffi::OsString;
use std::path::PathBuf;
use std::error::Error;

use structopt::StructOpt;

mod setup;
mod cli_user_choices;
mod wizard;
mod skip_wizard;
mod mytui;
mod export_csv;
mod export_txt;
mod export_je;
mod export_all;
mod tests;


#[derive(StructOpt, Debug)]
#[structopt(name = "cryptools")]
pub struct Cli {

    #[structopt(flatten)]
    flags: Flags,

    #[structopt(flatten)]
    opts: Options,

    /// File to be imported.  (Currently, the only supported date format is %m/%d/%y.)
    #[structopt(name = "file", parse(from_os_str))]
    file_to_import: Option<PathBuf>,
}

#[derive(StructOpt, Debug)]
pub struct Flags {

    /// User is instructing the program to skip the data entry wizard.
    /// When set, program will error without required command-line args.
    #[structopt(name = "accept args", short = "a", long = "accept")]
    accept_args: bool,

    /// This flag will suppress the printing of "all" reports, except that it will trigger the
    /// production of a txt file containing an accounting journal entry for every transaction.
    /// Individual account and transaction reports may still be printed via the print_menu
    /// with the -p flag. The journal entry report is only suitable for non-like-kind activity.
    #[structopt(name = "journal entries", short, long = "journal-entries")]
    journal_entries_only: bool,

    /// This will cause the program to expect the txDate field in the file_to_import to use the format
    /// YYYY-MM-dd or YY-MM-dd (or YYYY/MM/dd or YY/MM/dd, depending on the date-separator option)
    /// instead of the default US-style MM-dd-YYYY or MM-dd-YY (or MM/dd/YYYY or MM/dd/YY, depending on the
    /// date separator option).
    #[structopt(name = "date conforms to ISO 8601", short = "i", long = "iso")]
    iso_date: bool,

    /// Once the import file has been fully processed, the user will be presented
    /// with a menu for manually selecting which reports to print/export.
    #[structopt(name = "print menu", short, long = "print-menu")]
    print_menu: bool,

    /// This will prevent the program from writing reports to files.
    /// This will be ignored if -a is not set (the wizard will always ask to output).
    #[structopt(name = "suppress reports", short, long = "suppress")]
    suppress_reports: bool,
}

#[derive(StructOpt, Debug)]
pub struct Options {

    /// Choose "h", "s", or "p" for hyphen, slash, or period (i.e., "-", "/", or ".") to indicate the separator
    /// character used in the file_to_import txDate column (i.e. 2017/12/31, 2017-12-31, or 2017.12.31).
    #[structopt(name = "date separator character", short, long = "date-separator", default_value = "h", parse(from_os_str))]
    date_separator: OsString,

    /// Home currency (currency in which all resulting reports are denominated).
    /// (Only available as a command line setting.)
    #[structopt(name = "home currency", short = "c", long = "currency", default_value = "USD", parse(from_os_str))]
    home_currency: OsString,

    /// Cutoff date through which like-kind exchange treatment should be applied.
    /// Please use %y-%m-%d (or %Y-%m-%d) format for like-kind cutoff date entry.
    #[structopt(name = "like-kind cutoff date", short, long = "lk-cutoff", parse(from_os_str))]
    lk_cutoff_date: Option<OsString>,

    /// Inventory costing method (in terms of lot selection, i.e., LIFO, FIFO, etc.).
    /// There are currently four options (1 through 4).
    #[structopt(name = "method number for lot selection", short, long, default_value = "1", parse(from_os_str), long_help =
    r"    1. LIFO according to the order the lot was created.
    2. LIFO according to the basis date of the lot.
    3. FIFO according to the order the lot was created.
    4. FIFO according to the basis date of the lot.
    ")]
    inv_costing_method: OsString,

    /// Output directory for exported reports.
    #[structopt(name = "output directory", short, long = "output", default_value = ".", parse(from_os_str))]
    output_dir_path: PathBuf,
}


fn main() -> Result<(), Box<dyn Error>> {

    let args = Cli::from_args();

    println!(
    "
    Hello, crypto-folk!  Welcome to cryptools!

    This software will import your csv file's ledger of cryptocurrency transactions.
    It will then process it by creating 'lots' and posting 'movements' to those lots.
    Along the way, it will keep track of income, expenses, gains, and losses.

    Note: it is designed to import a full history. Gains and losses may be incorrect otherwise.
    ");

    let (input_file_path, settings) = setup::run_setup(args)?;

    let (
        account_map,
        raw_acct_map,
        action_records_map,
        transactions_map,
    ) = crptls::core_functions::import_and_process_final(input_file_path, &settings)?;

    let mut should_export_all = settings.should_export;
    let present_print_menu_tui = settings.print_menu;
    let print_journal_entries_only = settings.journal_entry_export;

    if present_print_menu_tui { should_export_all = false }
    if print_journal_entries_only { should_export_all = false }

    if should_export_all {

        export_all::export(
            &settings,
            &action_records_map,
            &raw_acct_map,
            &account_map,
            &transactions_map
        )?;
    }

    if print_journal_entries_only {

        if !settings.lk_treatment_enabled {
            export_je::prepare_non_lk_journal_entries(
                &settings,
                &action_records_map,
                &raw_acct_map,
                &account_map,
                &transactions_map,
            )?;
        }
    }

    if present_print_menu_tui {

        mytui::print_menu_tui::print_menu_tui(
            &settings,
            &action_records_map,
            &raw_acct_map,
            &account_map,
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
