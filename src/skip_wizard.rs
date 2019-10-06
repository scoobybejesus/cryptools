// Copyright (c) 2017-2019, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools/blob/master/LEGAL.txt

use std::error::Error;
use std::collections::{HashMap};

use crate::cli_user_choices;
use crate::core_functions::{self, LikeKindSettings, ImportProcessParameters};
use crate::account::{Account, RawAccount};
use crate::transaction::{Transaction, ActionRecord};

pub(crate) fn skip_wizard(
    non_excl_args: super::NonExclusiveToImportWizardArgs,
    excl_args: super::ExclusiveToImportWizardArgs
) -> Result<(
    HashMap<u16, Account>,
    HashMap<u16, RawAccount>,
    HashMap<u32, ActionRecord>,
    HashMap<u32, Transaction>,
    Option<LikeKindSettings>,
    ImportProcessParameters,
    bool
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

    let settings = ImportProcessParameters {
        export_path: excl_args.output_dir_path,
        home_currency: non_excl_args.home_currency,
        costing_method: costing_method_choice,
        enable_like_kind_treatment: like_kind_election,
        lk_cutoff_date_string: like_kind_cutoff_date_string,
        date_separator: non_excl_args.date_separator,
        iso_date_style: non_excl_args.iso_date
    };

    let (
        account_map1,
        raw_acct_map1,
        action_records_map1,
        transactions_map1,
        like_kind_settings1
    ) = core_functions::import_and_process_final(non_excl_args.file_to_import, &settings)?;

    let should_export = !excl_args.suppress_reports;

    Ok((account_map1, raw_acct_map1, action_records_map1, transactions_map1, like_kind_settings1, settings, should_export))
}