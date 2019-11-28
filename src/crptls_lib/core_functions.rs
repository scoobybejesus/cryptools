// Copyright (c) 2017-2019, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools/blob/master/LEGAL.txt

use std::path::PathBuf;
use std::error::Error;
use std::fs::File;
use std::collections::{HashMap};

use chrono::NaiveDate;

use crate::crptls_lib::account::{Account, RawAccount, Lot};
use crate::crptls_lib::transaction::{Transaction, ActionRecord};
use crate::crptls_lib::csv_import_accts_txns;
use crate::crptls_lib::import_cost_proceeds_etc;
use crate::crptls_lib::create_lots_mvmts;
use crate::crptls_lib::costing_method::InventoryCostingMethod;


/// `ImportProcessParameters` are determined from command-line args and/or wizard input from the user.
/// They are the settings that allow the software to carry out the importing-from-csv of
/// `Account`s and `Transaction`s, creation of `Lot`s and `Movement`s, addition of cost basis and proceeds
/// to `Movement`s, and application of like-kind treatment, in a specific and automated fashion.
pub struct ImportProcessParameters {
    pub export_path: PathBuf,
    pub home_currency: String,
    pub lk_treatment_enabled: bool,
    /// NaiveDate either from "1-1-1" (default and not to be used) or the actual date chosen (or passed in via Cli option)
    pub lk_cutoff_date: NaiveDate,
    pub lk_basis_date_preserved: bool,
    pub costing_method: InventoryCostingMethod,
    pub input_file_date_separator: String,
    pub input_file_has_iso_date_style: bool,
    pub should_export: bool,
    pub print_menu: bool,
    pub journal_entry_export: bool,
}

pub fn import_and_process_final(
    input_file_path: PathBuf,
    settings: &ImportProcessParameters,
) -> Result<(
    HashMap<u16, Account>,
    HashMap<u16, RawAccount>,
    HashMap<u32, ActionRecord>,
    HashMap<u32, Transaction>,
), Box<dyn Error>> {

    let mut transactions_map: HashMap<u32, Transaction> = HashMap::new();
    let mut action_records_map: HashMap<u32, ActionRecord> = HashMap::new();
    let mut raw_account_map: HashMap<u16, RawAccount> = HashMap::new();
    let mut account_map: HashMap<u16, Account> = HashMap::new();
    let mut _lot_map: HashMap<(RawAccount, u32), Lot> = HashMap::new();

    match import_from_csv(
        input_file_path,
        &mut transactions_map,
        &mut action_records_map,
        &mut raw_account_map,
        &mut account_map,
        &settings.input_file_date_separator,
        settings.input_file_has_iso_date_style,
    ) {
        Ok(()) => { println!("Successfully imported csv file."); }
        Err(err) => {
            println!("\nFailed to import accounts and transactions from CSV.");
            println!("{}", err);

            return Err(err)
        }
    };

    pub(crate) fn import_from_csv(
        import_file_path: PathBuf,
        transactions_map: &mut HashMap<u32, Transaction>,
        action_records: &mut HashMap<u32, ActionRecord>,
        raw_acct_map: &mut HashMap<u16, RawAccount>,
        acct_map: &mut HashMap<u16, Account>,
        date_separator: &str,
        iso_date_style: bool,
    ) -> Result<(), Box<dyn Error>> {

        let file = File::open(import_file_path)?; println!("CSV ledger file opened successfully.\n");

        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(file);

        csv_import_accts_txns::import_accounts(&mut rdr, raw_acct_map, acct_map)?;

        csv_import_accts_txns::import_transactions(
            &mut rdr,
            transactions_map,
            action_records,
            date_separator,
            iso_date_style,
        )?;

        Ok(())
    }

    transactions_map = create_lots_mvmts::create_lots_and_movements(
        transactions_map,
        &action_records_map,
        &raw_account_map,
        &account_map,
        &settings.home_currency.as_str(),
        &settings.costing_method,
        settings.lk_treatment_enabled,
        settings.lk_cutoff_date,
        settings.lk_basis_date_preserved,
        // &mut lot_map,
    )?;

    println!("  Successfully created lots and movements.");

    import_cost_proceeds_etc::add_cost_basis_to_movements(
        &settings,
        &action_records_map,
        &raw_account_map,
        &account_map,
        &transactions_map
    )?;

    println!("  Successfully added cost basis to movements.");

    import_cost_proceeds_etc::add_proceeds_to_movements(
        &action_records_map,
        &raw_account_map,
        &account_map,
        &transactions_map
    )?;

    println!("  Successfully added proceeds to movements.");


    if settings.lk_treatment_enabled {

        println!(" Applying like-kind treatment for cut-off date: {}.", settings.lk_cutoff_date);

        import_cost_proceeds_etc::apply_like_kind_treatment(
            &settings,
            &action_records_map,
            &raw_account_map,
            &account_map,
            &transactions_map
        )?;

        println!("  Successfully applied like-kind treatment.");
    }

    Ok((account_map, raw_account_map, action_records_map, transactions_map))
}
