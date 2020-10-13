// Copyright (c) 2017-2019, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools/blob/master/LEGAL.txt

use std::path::PathBuf;
use std::error::Error;
use std::process;
use std::env;

use chrono::NaiveDate;
use dotenv;

use crptls::core_functions::ImportProcessParameters;
use crptls::costing_method::InventoryCostingMethod;

use crate::cli_user_choices;
use crate::skip_wizard;
use crate::wizard;


pub fn get_env() -> Result<super::Cfg, Box<dyn Error>> {

    match dotenv::dotenv() {
        Ok(_path) => {println!("Setting environment variables from .env file.")},
        Err(_e) => println!("Did not find .env file.")
    }

    let iso_date: bool = match env::var("ISO_DATE") {
        Ok(val) => {
            if val == "1" || val.to_lowercase() == "true" {
                true
            } else {
                false
            }
        },
        Err(_e) => false,
    };

    let date_separator: String = match env::var("DATE_SEPARATOR") {
        Ok(val) => {
            println!("  Found DATE_SEPARATOR env var: {}", val);
            val.to_lowercase()},
        Err(_e) => {
            println!("  Using default date separator (hyphen).");
            "h".to_string()},
    };

    let home_currency = match env::var("HOME_CURRENCY") {
        Ok(val) => {
            println!("  Found HOME_CURRENCY env var: {}", val);
            val.to_uppercase()},
        Err(_e) => {
            println!("  Using default home currency (USD).");
            "USD".to_string()},
    };

    let lk_cutoff_date = match env::var("LK_CUTOFF_DATE") {
        Ok(val) => {
            println!("  Found LK_CUTOFF_DATE env var: {}", val);
            Some(val)},
        Err(_e) => None,
    };
    
    let inv_costing_method = match env::var("INV_COSTING_METHOD") {
        Ok(val) => {
            println!("  Found INV_COSTING_METHOD env var: {}", val);
            val},
        Err(_e) => {
            println!("  Using default inventory costing method (LIFO by lot creation date).");
            "1".to_string()},
    };

    let cfg = super::Cfg {
        iso_date,
        date_separator,
        home_currency,
        lk_cutoff_date,
        inv_costing_method,
    };

    Ok(cfg)
}

// These fields are subject to change by the user if they use the wizard
pub struct ArgsForImportVarsTBD {
    pub inv_costing_method_arg: String,
    pub lk_cutoff_date_arg: Option<String>,
    pub output_dir_path: PathBuf,
    pub suppress_reports: bool,
}

pub (crate) fn run_setup(cmd_args: super::Cli, cfg: super::Cfg) -> Result<(PathBuf, ImportProcessParameters), Box<dyn Error>> {

    let date_separator = match cfg.date_separator.as_str() {
        "h" => { "-" } 
        "s" => { "/" }
        "p" => { "." }
        _ => { println!("\nFATAL: The date-separator arg requires either an 'h', an 's', or a 'p'.\n"); process::exit(1) }
    };

    let input_file_path = match cmd_args.file_to_import {
        Some(file) => file,
        None => cli_user_choices::choose_file_for_import(cmd_args.accept_args)?
    };

    let wizard_or_not_args = ArgsForImportVarsTBD {
        inv_costing_method_arg: cfg.inv_costing_method,
        lk_cutoff_date_arg: cfg.lk_cutoff_date,
        output_dir_path: cmd_args.output_dir_path,
        suppress_reports: cmd_args.suppress_reports,
    };

    let(
        costing_method_choice,
        like_kind_election,
        like_kind_cutoff_date_string,
        should_export,
        output_dir_path,
     ) = wizard_or_not(cmd_args.accept_args, wizard_or_not_args)?;

    let like_kind_cutoff_date = if like_kind_election {
        NaiveDate::parse_from_str(&like_kind_cutoff_date_string, "%y-%m-%d")
            .unwrap_or_else(|_| NaiveDate::parse_from_str(&like_kind_cutoff_date_string, "%Y-%m-%d")
            .expect("Environment variable for LK_CUTOFF_DATE has an incorrect format. Program must abort. See .env.example."))
    } else { NaiveDate::parse_from_str(&"1-1-1", "%y-%m-%d").unwrap() };

    let settings = ImportProcessParameters {
        input_file_uses_iso_date_style: cfg.iso_date,
        input_file_date_separator: date_separator.to_string(),
        home_currency: cfg.home_currency.to_uppercase(),
        costing_method: costing_method_choice,
        lk_treatment_enabled: like_kind_election,
        lk_cutoff_date: like_kind_cutoff_date,
        lk_basis_date_preserved: true,  //  TODO
        should_export,
        export_path: output_dir_path,
        print_menu: cmd_args.print_menu,
        journal_entry_export: cmd_args.journal_entries_only,
    };

    Ok((input_file_path, settings))
}

fn wizard_or_not(accept_args: bool, args: ArgsForImportVarsTBD) -> Result<(
    InventoryCostingMethod,
    bool,
    String,
    bool,
    PathBuf,
), Box<dyn Error>> {

    let costing_method_choice;
    let like_kind_election;
    let like_kind_cutoff_date_string;
    let should_export;
    let output_dir_path;

    if !accept_args {

        let (
            costing_method_choice1,
            like_kind_election1,
            like_kind_cutoff_date_string1,
            should_export1,
            output_dir_path1,
        ) = wizard::wizard(args)?;

        costing_method_choice = costing_method_choice1;
        like_kind_election = like_kind_election1;
        like_kind_cutoff_date_string = like_kind_cutoff_date_string1;
        should_export = should_export1;
        output_dir_path = output_dir_path1;

    } else {

        let (
            costing_method_choice1,
            like_kind_election1,
            like_kind_cutoff_date_string1,
            should_export1,
            output_dir_path1,
        ) = skip_wizard::skip_wizard(args)?;

        costing_method_choice = costing_method_choice1;
        like_kind_election = like_kind_election1;
        like_kind_cutoff_date_string = like_kind_cutoff_date_string1;
        should_export = should_export1;
        output_dir_path = output_dir_path1;

    }

    Ok((costing_method_choice, like_kind_election, like_kind_cutoff_date_string, should_export, output_dir_path))
}