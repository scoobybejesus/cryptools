// Copyright (c) 2017-2023, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools/blob/master/LEGAL.txt

use std::error::Error;
use std::collections::HashMap;

use crptls::transaction::{Transaction, ActionRecord};
use crptls::account::{Account, RawAccount};
use crptls::core_functions::ImportProcessParameters;
use ratatui::widgets::ListState;

use crate::export::{export_csv, export_je, export_txt};

pub (crate) const REPORTS: [&'static str; 11] = [
    "1. CSV: Account Sums",
    "2. CSV: Account Sums (Non-zero only)",
    "3. CSV: Account Sums (Orig. basis vs like-kind basis)",
    "4. CSV: Transactions by movement (every movement)",
    "5. CSV: Transactions by movement (summarized by long-term/short-term)",
    "6. CSV: Transactions by movement (every movement, w/ orig. and like-kind basis",
    "7. CSV: Transactions summary by LT/ST for Form 8949",
    "8. TXT: Accounts by lot (every movement)",
    "9. TXT: Accounts by lot (every lot balance)",
    "10. TXT: Accounts by lot (every non-zero lot balance)",
    "11. TXT: Bookkeeping journal entries",
];

pub struct StatefulList<I> {
    pub items: Vec<I>,
    pub state: ListState,
}

impl<T> StatefulList<T> {

    fn new(items: Vec<T>) -> StatefulList<T> {
        StatefulList { items, state: ListState::default() }
    }

    fn select_previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn select_next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

}

pub struct PrintWindow<'a> {
    pub title: &'a str,
    pub should_quit: bool,
    pub tasks: StatefulList<&'a str>,
    pub to_print_by_idx: Vec<usize>,
    pub to_print_by_title: Vec<&'a str>,
}

impl<'a> PrintWindow<'a> {

    pub fn new(title: &'a str) -> PrintWindow<'a> {
        let mut tasks = StatefulList::new(REPORTS.to_vec());
        tasks.state.select(Some(0));

        PrintWindow {
            title,
            should_quit: false,
            tasks,
            to_print_by_idx: Vec::with_capacity(REPORTS.len()),
            to_print_by_title: Vec::with_capacity(REPORTS.len()),
        }
    }

    pub fn on_up(&mut self) {
        self.tasks.select_previous();
    }

    pub fn on_down(&mut self) {
        self.tasks.select_next();
    }

    pub fn on_key(&mut self, c: char) -> Result<(), Box<dyn Error>> {

        match c {

            'q' => {
                self.to_print_by_idx = Vec::with_capacity(0);
                self.should_quit = true;
            }
            'p' => {
                self.should_quit = true;
            }
            'x' => {
                let selected = self.tasks.state.selected().unwrap();
                if self.to_print_by_idx.contains(&selected) {} else {
                    self.to_print_by_idx.push(selected);
                    self.to_print_by_title.push(self.tasks.items[selected])
                }
                Self::change_vecs_to_chrono_order(&mut self.to_print_by_idx, &mut self.to_print_by_title);
                self.tasks.select_next();
            }
            'd' => {
                let selected_idx = self.tasks.state.selected().unwrap();
                self.to_print_by_idx.retain(|&x| x != selected_idx );
                let selected_str = self.tasks.items[selected_idx];
                self.to_print_by_title.retain(|&x| x != selected_str );
                self.tasks.select_previous();
            }
            _ => {}
        }
        Ok(())
    }

    fn change_vecs_to_chrono_order(vec: &mut Vec<usize>, strvec: &mut Vec<&str>) {

        let length = vec.len();

        for _ in 0..length {
            for j in 0..length-1 {
                if vec[j] > vec[j+1] {
                    vec.swap(j, j+1);
                    strvec.swap(j, j+1)
                }
            }
        }
    }
}

pub fn export(
    app: &PrintWindow,
    settings: &ImportProcessParameters,
    raw_acct_map: &HashMap<u16, RawAccount>,
    account_map: &HashMap<u16, Account>,
    action_records_map: &HashMap<u32, ActionRecord>,
    transactions_map: &HashMap<u32, Transaction>,
) -> Result<(), Box<dyn Error>> {

    let reports = REPORTS.to_vec();

    println!("Attempting to export:");

    if app.to_print_by_idx.is_empty() {
        println!("  None selected.");
        return Ok(())
    }

    for report_idx in app.to_print_by_idx.iter() {

        println!("    {}", reports[*report_idx]);

        match report_idx + 1 {

            1 => {
                export_csv::_1_account_sums_to_csv(
                    &settings,
                    &raw_acct_map,
                    &account_map
                );
            }
            2 => {
                export_csv::_2_account_sums_nonzero_to_csv(
                    &settings,
                    &raw_acct_map,
                    &account_map,
                );
            }
            3 => {
                export_csv::_3_account_sums_to_csv_with_orig_basis(
                    &settings,
                    &raw_acct_map,
                    &account_map
                );
            }
            4 => {
                export_csv::_4_transaction_mvmt_detail_to_csv(
                    &settings,
                    &raw_acct_map,
                    &account_map,
                    &action_records_map,
                    &transactions_map
                )?;
            }
            5 => {
                export_csv::_5_transaction_mvmt_summaries_to_csv(
                    &settings,
                    &raw_acct_map,
                    &account_map,
                    &action_records_map,
                    &transactions_map
                )?;
            }
            6 => {
                export_csv::_6_transaction_mvmt_detail_to_csv_w_orig(
                    &settings,
                    &raw_acct_map,
                    &account_map,
                    &action_records_map,
                    &transactions_map
                )?;
            }
            7 => {
                export_csv::_7_gain_loss_8949_to_csv(
                    &settings,
                    &raw_acct_map,
                    &account_map,
                    &action_records_map,
                    &transactions_map
                )?;
            }

            8 => {
                export_txt::_1_account_lot_detail_to_txt(
                    &settings,
                    &raw_acct_map,
                    &account_map,
                    &action_records_map,
                    &transactions_map,
                )?;
            }
            9 => {
                export_txt::_2_account_lot_summary_to_txt(
                    &settings,
                    &raw_acct_map,
                    &account_map,
                )?;
            }
            10 => {
                export_txt::_3_account_lot_summary_non_zero_to_txt(
                    &settings,
                    &raw_acct_map,
                    &account_map,
                )?;
            }
            11 => {
                if !settings.lk_treatment_enabled {
                    export_je::prepare_non_lk_journal_entries(
                        &settings,
                        &raw_acct_map,
                        &account_map,
                        &action_records_map,
                        &transactions_map,
                    )?;
                } else {
                    println!("       *Skipping non-like-kind report: {}", reports[*report_idx]);
                }
            }
            _ => {}
        }
    }
    println!("Successfully exported.");
    Ok(())
}