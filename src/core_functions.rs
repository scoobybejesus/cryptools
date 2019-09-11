// Copyright (c) 2017-2019, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools-rs/blob/master/LEGAL.txt

use std::path::PathBuf;
use std::error::Error;
use std::fs::File;
use std::collections::{HashMap};
use std::fmt;

use chrono::NaiveDate;
use structopt::StructOpt;

use crate::account::{Account, RawAccount, Lot};
use crate::transaction::{Transaction, ActionRecord};
use crate::csv_import_accts_txns;
use crate::import_cost_proceeds_etc;
use crate::create_lots_mvmts;

#[derive(Clone, Debug, PartialEq, StructOpt)]
pub enum InventoryCostingMethod {
    /// 1. LIFO according to the order the lot was created.
    LIFObyLotCreationDate,
    /// 2. LIFO according to the basis date of the lot.
    LIFObyLotBasisDate,
    /// 3. FIFO according to the order the lot was created.
    FIFObyLotCreationDate,
    /// 4. FIFO according to the basis date of the lot.
    FIFObyLotBasisDate,
}

impl fmt::Display for InventoryCostingMethod {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
           InventoryCostingMethod::LIFObyLotCreationDate => write!(f, "LIFO by lot creation date"),
           InventoryCostingMethod::LIFObyLotBasisDate => write!(f, "LIFO by lot basis date"),
           InventoryCostingMethod::FIFObyLotCreationDate => write!(f, "FIFO by lot creation date"),
           InventoryCostingMethod::FIFObyLotBasisDate => write!(f, "FIFO by lot basis date"),
       }
    }
}

#[derive(Clone)]
pub struct LikeKindSettings {
    pub like_kind_cutoff_date: NaiveDate,
    pub like_kind_basis_date_preserved: bool,
}

pub struct ImportProcessParameters {
    pub export_path: PathBuf,
    pub home_currency: String,
    pub enable_like_kind_treatment: bool,
    pub costing_method: InventoryCostingMethod,
    pub lk_cutoff_date_string: String,
}

pub fn import_and_process_final(
    input_file_path: PathBuf,
    settings: &ImportProcessParameters,
) -> Result<(
    HashMap<u16, Account>,
    HashMap<u16, RawAccount>,
    HashMap<u32, ActionRecord>,
    HashMap<u32, Transaction>,
    Option<LikeKindSettings>
), Box<dyn Error>> {

    let mut transactions_map: HashMap<u32, Transaction> = HashMap::new();
    let mut action_records_map: HashMap<u32, ActionRecord> = HashMap::new();
    let mut raw_account_map: HashMap<u16, RawAccount> = HashMap::new();
    let mut account_map: HashMap<u16, Account> = HashMap::new();
    let mut lot_map: HashMap<(RawAccount, u32), Lot> = HashMap::new();

    match import_from_csv(
        input_file_path,
        &mut transactions_map,
        &mut action_records_map,
        &mut raw_account_map,
        &mut account_map
    ) {
        Ok(()) => { println!("Successfully imported csv file."); }
        Err(err) => {
            println!("\nFailed to import accounts and transactions from CSV.");
            println!("{}", err);

            return Err(err)
        }
    };

    pub fn import_from_csv(
        import_file_path: PathBuf,
        transactions_map: &mut HashMap<u32, Transaction>,
        action_records: &mut HashMap<u32, ActionRecord>,
        raw_acct_map: &mut HashMap<u16, RawAccount>,
        acct_map: &mut HashMap<u16, Account>,
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
        )?;

        Ok(())
    }
    // println!("like_kind_cutoff_date is: {}...", like_kind_cutoff_date);

    let likekind_settings: Option<LikeKindSettings> = if settings.enable_like_kind_treatment {

        let like_kind_cutoff_date = &settings.lk_cutoff_date_string;

        Some(
            LikeKindSettings {
                like_kind_cutoff_date: NaiveDate::parse_from_str(&like_kind_cutoff_date, "%y-%m-%d")
                    .unwrap_or(NaiveDate::parse_from_str(&like_kind_cutoff_date, "%Y-%m-%d")
                    .expect("Found date string with improper format")),
                like_kind_basis_date_preserved: true,
            }
        )
    } else {
        None
    };

    transactions_map = create_lots_mvmts::create_lots_and_movements(
        transactions_map,
        &settings,
        &likekind_settings,
        &action_records_map,
        &mut raw_account_map,
        &mut account_map,
        &mut lot_map,
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


    if let Some(lk_settings) = &likekind_settings {

        let cutoff_date = lk_settings.like_kind_cutoff_date;
        println!(" Applying like-kind treatment for cut-off date: {}.", cutoff_date);

        import_cost_proceeds_etc::apply_like_kind_treatment(
            cutoff_date,
            &settings,
            &action_records_map,
            &raw_account_map,
            &account_map,
            &transactions_map
        )?;

        println!("  Successfully applied like-kind treatment.");
    };

    Ok((account_map, raw_account_map, action_records_map, transactions_map, likekind_settings))
}
