// Copyright (c) 2017-2019, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools/blob/master/LEGAL.txt

use std::path::PathBuf;
use std::error::Error;

use crate::cli_user_choices;
use crate::core_functions::{InventoryCostingMethod};
use crate::setup::{WizardMaybeArgs};

pub(crate) fn skip_wizard(excl_args: WizardMaybeArgs) -> Result<(
    InventoryCostingMethod,
    bool,
    String,
    bool,
    PathBuf,
), Box<dyn Error>> {

    let costing_method_choice = cli_user_choices::inv_costing_from_cmd_arg(excl_args.inv_costing_method_arg)?;

    let like_kind_election;
    let like_kind_cutoff_date_string: String;

    if let Some(date) = excl_args.lk_cutoff_date_arg {
        like_kind_election = true;
        like_kind_cutoff_date_string = date.into_string().unwrap();
    } else {
        like_kind_election = false;
        like_kind_cutoff_date_string = "1-1-1".to_string();
    };

    let should_export = !excl_args.suppress_reports;

    Ok((costing_method_choice, like_kind_election, like_kind_cutoff_date_string, should_export, excl_args.output_dir_path))
}