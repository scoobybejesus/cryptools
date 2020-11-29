// Copyright (c) 2017-2019, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools/blob/master/LEGAL.txt

use std::path::PathBuf;
use std::error::Error;

use std::collections::{HashMap};

use chrono::NaiveDate;

use crate::crptls_lib::account::{Account, RawAccount, Lot};
use crate::crptls_lib::transaction::{Transaction, ActionRecord};
use crate::crptls_lib::csv_import_accts_txns;
use crate::crptls_lib::import_cost_proceeds_etc;
use crate::crptls_lib::create_lots_mvmts;
use crate::crptls_lib::costing_method::InventoryCostingMethod;


/// `ImportProcessParameters` are determined from command-line args, environment variables, and/or wizard input from the user.
/// They are the settings that allow the software to carry out the importing-from-csv of
/// `Account`s and `Transaction`s, creation of `Lot`s and `Movement`s, addition of cost basis and proceeds
/// to `Movement`s, and application of like-kind treatment, in a specific and automated fashion.
pub struct ImportProcessParameters {
    pub input_file_date_separator: String,
    pub input_file_uses_iso_date_style: bool,
    pub home_currency: String,
    pub costing_method: InventoryCostingMethod,
    pub lk_treatment_enabled: bool,
    /// NaiveDate either from "1-1-1" (default and not to be used) or the actual date chosen (or passed in via env var)
    pub lk_cutoff_date: NaiveDate,
    pub lk_basis_date_preserved: bool,
    pub should_export: bool,
    pub export_path: PathBuf,
    pub print_menu: bool,
    pub journal_entry_export: bool,
}

pub fn import_and_process_final(
    input_file_path: PathBuf,
    settings: &ImportProcessParameters,
) -> Result<(
    HashMap<u16, RawAccount>,
    HashMap<u16, Account>,
    HashMap<u32, ActionRecord>,
    HashMap<u32, Transaction>,
), Box<dyn Error>> {

    let mut transactions_map: HashMap<u32, Transaction> = HashMap::new();
    let mut action_records_map: HashMap<u32, ActionRecord> = HashMap::new();
    let mut raw_account_map: HashMap<u16, RawAccount> = HashMap::new();
    let mut account_map: HashMap<u16, Account> = HashMap::new();
    let mut _lot_map: HashMap<(RawAccount, u32), Lot> = HashMap::new();

    csv_import_accts_txns::import_from_csv(
        input_file_path,
        settings,
        &mut raw_account_map,
        &mut account_map,
        &mut action_records_map,
        &mut transactions_map,
    )?;

    println!("  Successfully imported csv file.");
    println!("Processing the data...");

    transactions_map = create_lots_mvmts::create_lots_and_movements(
        &settings,
        &raw_account_map,
        &account_map,
        &action_records_map,
        transactions_map,
        // &mut lot_map,
    )?;

    println!("  Created lots and movements.");

    import_cost_proceeds_etc::add_cost_basis_to_movements(
        &settings,
        &raw_account_map,
        &account_map,
        &action_records_map,
        &transactions_map
    )?;

    println!("  Added cost basis to movements.");

    import_cost_proceeds_etc::add_proceeds_to_movements(
        &raw_account_map,
        &account_map,
        &action_records_map,
        &transactions_map
    )?;

    println!("  Added proceeds to movements.");

    if settings.lk_treatment_enabled {

        println!(" Applying like-kind treatment through cut-off date: {}.", settings.lk_cutoff_date);

        import_cost_proceeds_etc::apply_like_kind_treatment(
            &settings,
            &raw_account_map,
            &account_map,
            &action_records_map,
            &transactions_map
        )?;

        println!("  Successfully applied like-kind treatment.");
    }

    Ok((raw_account_map, account_map, action_records_map, transactions_map))
}
