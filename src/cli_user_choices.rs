// Copyright (c) 2017-2019, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools/blob/master/LEGAL.txt

use std::error::Error;
use std::io::{self, BufRead};
use std::process;
use std::path::PathBuf;
use std::ffi::OsString;

use chrono::NaiveDate;
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::{CompletionType, Config, Context, EditMode, Editor, Helper};
use rustyline::config::OutputStreamType;
use rustyline::hint::{Hinter};
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter};

use crptls::costing_method::InventoryCostingMethod;
use crptls::string_utils;


pub fn choose_file_for_import(flag_to_accept_cli_args: bool) -> Result<PathBuf, Box<dyn Error>> {

    if flag_to_accept_cli_args {
        println!("WARN: Flag to 'accept args' was set, but 'file' is missing.\n");
    }

    println!("Please input a file (absolute or relative path) to import: ");

    let (file_string, has_tilde) = _get_path()?;

    if has_tilde {
        choose_file_for_import(flag_to_accept_cli_args)
    } else {
        Ok( PathBuf::from(file_string) )
    }
}

pub fn choose_export_dir() -> Result<PathBuf, Box<dyn Error>> {

    println!("Please input a file path for exports: ");

    let (file_string, has_tilde) = _get_path()?;

    if has_tilde {
        choose_export_dir()
    } else {
        Ok( PathBuf::from(file_string) )
    }
}

fn _get_path() -> Result<(String, bool), Box<dyn Error>> {

    struct MyHelper {
        completer: FilenameCompleter,
        colored_prompt: String,
    }

    impl Completer for MyHelper {

        type Candidate = Pair;

        fn complete(
            &self,
            line: &str,
            pos: usize,
            ctx: &Context<'_>,
        ) -> Result<(usize, Vec<Pair>), ReadlineError> {
            self.completer.complete(line, pos, ctx)
        }
    }

    impl Hinter for MyHelper {}
    impl Highlighter for MyHelper {}
    impl Helper for MyHelper {}

    let h = MyHelper {
        completer: FilenameCompleter::new(),
        colored_prompt: "".to_owned(),
    };

    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::Circular)
        .edit_mode(EditMode::Vi)
        .output_stream(OutputStreamType::Stdout)
        .build();

    let count = 1;
    let mut rl = Editor::with_config(config);
    let p = format!("{}> ", count);
    rl.set_helper(Some(h));
    rl.helper_mut().unwrap().colored_prompt = format!("\x1b[1;32m{}\x1b[0m", p);
    let readline = rl.readline(">> ");

    fn begins_with_tilde(unchecked_path: &String) -> bool {
        match unchecked_path.find("~") {
            Some(0) => return true,
            _ => return false
        }
    }

    match readline {
        Ok(line) => {
            println!("");
            let has_tilde = begins_with_tilde(&line);
            if has_tilde {
                println!("Unfortunately, the tilde '~' cannot be used as a shortcut for your home directory.\n");
            }
            Ok((line, has_tilde))
        },
        Err(err) => {
            println!("Error during Rustyline: {:?}", err);
            process::exit(1)
        }
    }
}

pub fn choose_inventory_costing_method(cmd_line_arg: OsString) -> Result<InventoryCostingMethod, Box<dyn Error>> {

    println!("Choose the lot inventory costing method. [Default/Chosen: {:?}]", cmd_line_arg);
    println!("1. LIFO according to the order the lot was created.");
    println!("2. LIFO according to the basis date of the lot.");
    println!("3. FIFO according to the order the lot was created.");
    println!("4. FIFO according to the basis date of the lot.");

    let method = _costing_method(cmd_line_arg)?;

    fn _costing_method(cmd_line_arg: OsString) -> Result<InventoryCostingMethod, Box<dyn Error>> {

        let mut input = String::new();
        let stdin = io::stdin();
        stdin.lock().read_line(&mut input).expect("Failed to read stdin");

        match input.trim() { // Without .trim(), there's a hidden \n or something preventing the match
            "" => Ok(inv_costing_from_cmd_arg(cmd_line_arg)?),
            "1" => Ok(InventoryCostingMethod::LIFObyLotCreationDate),
            "2" => Ok(InventoryCostingMethod::LIFObyLotBasisDate),
            "3" => Ok(InventoryCostingMethod::FIFObyLotCreationDate),
            "4" => Ok(InventoryCostingMethod::FIFObyLotBasisDate),
            _   => { println!("Invalid choice.  Please enter a valid choice."); _costing_method(cmd_line_arg) }
        }
    }

    Ok(method)
}
pub fn inv_costing_from_cmd_arg(arg: OsString) -> Result<InventoryCostingMethod, &'static str> {

    let inv_costing_arg = match arg.into_string().expect("Could not convert OsString to String.").trim() {
        "1" => {"1"} "2" => {"2"} "3" => {"3"} "4" => {"4"}
        _ => { println!("WARN: Invalid command line arg passed for 'inv_costing_method'. Using default."); "1" }
    };

    match inv_costing_arg {
        "1" => Ok(InventoryCostingMethod::LIFObyLotCreationDate),
        "2" => Ok(InventoryCostingMethod::LIFObyLotBasisDate),
        "3" => Ok(InventoryCostingMethod::FIFObyLotCreationDate),
        "4" => Ok(InventoryCostingMethod::FIFObyLotBasisDate),
        _   => { Err("Invalid input parameter") }   //  Impossible code path
    }
}

pub(crate) fn elect_like_kind_treatment(cutoff_date_arg: &mut Option<String>) -> Result<(bool, String), Box<dyn Error>> {

    match cutoff_date_arg.clone() {

        Some(mut cutoff_date_arg) => {

            let provided_date = NaiveDate::parse_from_str(&cutoff_date_arg, "%y-%m-%d")
                .unwrap_or(NaiveDate::parse_from_str(&cutoff_date_arg, "%Y-%m-%d")
                .unwrap_or_else(|e| {
                    println!("\nWARN: Date entered after -l command line arg (like-kind cutoff date) has an invalid format.");
                    second_date_try_from_user(&mut cutoff_date_arg).unwrap()
                } ) );

            println!("\nUse like-kind exchange treatment through {}? [Y/n/c] ('c' to 'change') ", provided_date);

            let (election, date_string) = _elect_like_kind_arg(&cutoff_date_arg, provided_date)?;

            fn _elect_like_kind_arg(cutoff_date_arg: &String, provided_date: NaiveDate) -> Result<(bool, String), Box<dyn Error>> {

                let mut input = String::new();
                let stdin = io::stdin();
                stdin.lock().read_line(&mut input)?;

                match input.trim().to_ascii_lowercase().as_str() {

                    "y" | "ye" | "yes" | "" => {

                        println!("   Using like-kind treatment through {}.\n", provided_date);
                        Ok( (true, cutoff_date_arg.to_string()) )
                    },

                    "n" | "no" => {

                        println!("   Proceeding without like-kind treatment.\n");
                        Ok( (false, "1-1-1".to_string()) )
                    },

                    "c" | "change" => {

                        let input = change_or_choose_lk_date_by_user()?;
                        Ok( (true, input) )
                    },

                    _   => {

                        println!("Please respond with 'y', 'n', or 'c' (or 'yes' or 'no' or 'change').");
                        _elect_like_kind_arg(&cutoff_date_arg, provided_date)
                    }
                }
            }

            return Ok((election, date_string))
        }

        None => {
            println!("\nContinue without like-kind exchange treatment? [Y/n] ");

            let (election, date_string) = _no_elect_like_kind_arg()?;

            fn _no_elect_like_kind_arg() -> Result<(bool, String), Box<dyn Error>> {

                let mut input = String::new();
                let stdin = io::stdin();
                stdin.lock().read_line(&mut input)?;

                match input.trim().to_ascii_lowercase().as_str() {

                    "y" | "ye" | "yes" | "" => {

                        println!("   Proceeding without like-kind treatment.\n");
                        Ok( (false, "1-1-1".to_string()) )
                    },

                    "n" | "no" => {

                        let input = change_or_choose_lk_date_by_user()?;
                        Ok( (true, input) )
                    },

                    _   => { println!("Please respond with 'y' or 'n' (or 'yes' or 'no')."); _no_elect_like_kind_arg() }
                }
            }

            return Ok((election, date_string))
        }
    }


    fn change_or_choose_lk_date_by_user() -> Result<String, Box<dyn Error>> {

        println!("Please enter your desired like-kind exchange treatment cutoff date using ISO 8601 style date format.");

        let mut input = String::new();
        let stdin = io::stdin();
        stdin.lock().read_line(&mut input)?;
        string_utils::trim_newline(&mut input);

        let successfully_parsed_naive_date = test_naive_date_from_user_string(&mut input)?;

        println!("   Using like-kind treatment through {}.\n", successfully_parsed_naive_date);

        Ok(input)
    }

    fn test_naive_date_from_user_string(input: &mut String) -> Result<NaiveDate, Box<dyn Error>> {

        let successfully_parsed_naive_date = NaiveDate::parse_from_str(&input, "%y-%m-%d")
            .unwrap_or(NaiveDate::parse_from_str(&input, "%Y-%m-%d")
            .unwrap_or_else(|e| { second_date_try_from_user(input).unwrap() } ));

        Ok(successfully_parsed_naive_date)
    }

    fn second_date_try_from_user(input: &mut String) -> Result<NaiveDate, Box<dyn Error>> {

        println!("  You must use the format %y-%m-%d (e.g., 2009-06-01, 09-06-01, and 9-6-1 are all acceptably formatted).");

        let mut input2 = String::new();
        let stdin = io::stdin();
        stdin.lock().read_line(&mut input2)?;
        string_utils::trim_newline(&mut input2);
        *input = input2;

        let successfully_parsed_naive_date = NaiveDate::parse_from_str(&input, "%y-%m-%d")
            .unwrap_or(NaiveDate::parse_from_str(&input, "%Y-%m-%d")
            .unwrap_or_else(|e| { second_date_try_from_user(input).unwrap() } ));

        Ok(successfully_parsed_naive_date)
    }
}
