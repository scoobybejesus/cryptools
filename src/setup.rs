// Copyright (c) 2017-2019, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools/blob/master/LEGAL.txt

use std::ffi::OsString;
use std::path::PathBuf;
use std::error::Error;
use std::process;

use chrono::NaiveDate;

use crate::cli_user_choices;
use crate::core_functions::{ImportProcessParameters, InventoryCostingMethod};
use crate::skip_wizard;
use crate::wizard;


pub struct ArgsForImportVarsTBD {
    pub inv_costing_method_arg: OsString,
    pub lk_cutoff_date_arg: Option<OsString>,
    pub output_dir_path: PathBuf,
    pub suppress_reports: bool,
}

pub (crate) fn run_setup(args: super::Cli) -> Result<(PathBuf, ImportProcessParameters), Box<dyn Error>> {

    let date_separator = match args.opts.date_separator.into_string().unwrap().as_str() {
        "h" => { "-" }
        "s" => { "/" }
        "p" => { "." }
        _ => { println!("\nFATAL: The date-separator arg requires either an 'h', an 's', or a 'p'.\n"); process::exit(1) }
    };

    let input_file_path = match args.file_to_import {
        Some(file) => file,
        None => cli_user_choices::choose_file_for_import(args.flags.accept_args)?
    };

    let wizard_or_not_args = ArgsForImportVarsTBD {
        inv_costing_method_arg: args.opts.inv_costing_method,
        lk_cutoff_date_arg: args.opts.lk_cutoff_date,
        output_dir_path: args.opts.output_dir_path,
        suppress_reports: args.flags.suppress_reports,
    };

    let(
        costing_method_choice,
        like_kind_election,
        like_kind_cutoff_date_string,
        should_export,
        output_dir_path,
     ) = wizard_or_not(args.flags.accept_args, wizard_or_not_args)?;

    let like_kind_cutoff_date;

    if like_kind_election {
        like_kind_cutoff_date = NaiveDate::parse_from_str(&like_kind_cutoff_date_string, "%y-%m-%d")
            .unwrap_or(NaiveDate::parse_from_str(&like_kind_cutoff_date_string, "%Y-%m-%d")
            .expect("Command line date (like-kind cutoff option) has an incorrect format. Program must abort."));
    } else {
        like_kind_cutoff_date = NaiveDate::parse_from_str(&"1-1-1", "%y-%m-%d").unwrap();
    }

    let settings = ImportProcessParameters {
        home_currency: args.opts.home_currency.into_string().unwrap().to_uppercase(),
        input_file_has_iso_date_style: args.flags.iso_date,
        input_file_date_separator: date_separator.to_string(),
        costing_method: costing_method_choice,
        lk_treatment_enabled: like_kind_election,
        lk_cutoff_date: like_kind_cutoff_date,
        lk_basis_date_preserved: true,  //  TODO
        should_export: should_export,
        export_path: output_dir_path,
        print_menu: args.flags.print_menu,
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