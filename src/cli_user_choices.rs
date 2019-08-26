// Copyright (c) 2017-2019, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools-rs/LEGAL.txt

use std::error::Error;
use std::io::{self, BufRead};
use std::process;
use std::path::PathBuf;

use chrono::NaiveDate;
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::{CompletionType, Config, Context, EditMode, Editor, Helper};
use rustyline::config::OutputStreamType;
use rustyline::hint::{Hinter};
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter};
use crate::core_functions::InventoryCostingMethod;

use crate::utils;


pub fn choose_file_for_import() -> PathBuf {

    println!("Please input a file (absolute or relative path) to import: ");

    // PathBuf::from("/Users/scoob/Documents/Repos/cryptools-rs/private/RawTxForImport-pycleaned.csv")

    let file_str = _get_path();
    PathBuf::from(file_str.unwrap())
}

pub fn choose_export_dir() -> PathBuf {

    println!("Please input a file path for exports: ");

    // PathBuf::from("/Users/scoob/Documents/Testing/rust_exports/")

    let file_str = _get_path();
    PathBuf::from(file_str.unwrap())
}

fn _get_path() -> Result<(String), Box<Error>> {

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

    match readline {
        Ok(line) => {
            rl.add_history_entry(line.as_str());
            println!("");
            Ok(line)
        },
        Err(err) => {
            println!("Error during Rustyline: {:?}", err);
            process::exit(1)
        }
    }
}


// impl std::convert::From<OsStr> for InventoryCostingMethod {
//     fn from(osstr: OsStr) -> InventoryCostingMethod {
//         let osstring1 = OsString::from(Box::<osstr>);
//         let new_string = osstr.into_string().expect("Invalid input. Could not convert to string.");
//         let method = match new_string.trim() {
//             "1" => InventoryCostingMethod::LIFObyLotCreationDate,
//             "2" => InventoryCostingMethod::LIFObyLotBasisDate,
//             "3" => InventoryCostingMethod::FIFObyLotCreationDate,
//             "4" => InventoryCostingMethod::FIFObyLotBasisDate,
//             _   => { println!("Invalid choice.  Could not convert."); process::exit(1)
//             }
//         };
//         method
//     }
// }

pub fn choose_inventory_costing_method() -> InventoryCostingMethod {

    println!("Choose the lot inventory costing method. [Default: 1]");
    println!("1. LIFO according to the order the lot was created.");
    println!("2. LIFO according to the basis date of the lot.");
    println!("3. FIFO according to the order the lot was created.");
    println!("4. FIFO according to the basis date of the lot.");

    let method = match _costing_method() {
        Ok(x) => {x},
        Err(err) => { process::exit(1) }
    };

    fn _costing_method() -> Result<(InventoryCostingMethod), Box<Error>> {

        let mut input = String::new();
        let stdin = io::stdin();
        stdin.lock().read_line(&mut input).expect("Failed to read stdin");

        match input.trim() { // Without .trim(), there's a hidden \n or something preventing the match
            "1" | "" => Ok(InventoryCostingMethod::LIFObyLotCreationDate),
            "2" => Ok(InventoryCostingMethod::LIFObyLotBasisDate),
            "3" => Ok(InventoryCostingMethod::FIFObyLotCreationDate),
            "4" => Ok(InventoryCostingMethod::FIFObyLotBasisDate),
            _   => { println!("Invalid choice.  Please enter a valid number."); _costing_method() }
        }
    }

    method
}
pub fn inv_costing_from_cmd_arg(arg: String) -> InventoryCostingMethod {

    let method = match _costing_method(arg) {
        Ok(x) => {x},
        Err(err) => { process::exit(1) }
    };

    fn _costing_method(input: String) -> Result<(InventoryCostingMethod), Box<Error>> {

        match input.trim() { // Without .trim(), there's a hidden \n or something preventing the match
            "1" => Ok(InventoryCostingMethod::LIFObyLotCreationDate),
            "2" => Ok(InventoryCostingMethod::LIFObyLotBasisDate),
            "3" => Ok(InventoryCostingMethod::FIFObyLotCreationDate),
            "4" => Ok(InventoryCostingMethod::FIFObyLotBasisDate),
            _   => { println!("Invalid choice.  Please enter a valid number."); _costing_method(input) }
        }
    }

    method
}
pub fn elect_like_kind_treatment(cutoff_date_arg: &Option<String>) -> (bool, String) {
    let election: bool;
    let date: String;

    if cutoff_date_arg.is_some() {

        let provided_date = NaiveDate::parse_from_str(&cutoff_date_arg.clone().unwrap(), "%y-%m-%d")
            .unwrap_or(NaiveDate::parse_from_str(&cutoff_date_arg.clone().unwrap(), "%Y-%m-%d")
            .expect("Date entered as -c command line arg has an incorrect format."));

        println!("\nUse like-kind exchange treatment through {}? [Y/n/c] ('c' to 'change') ", provided_date);

        let (election, date) = match _elect_like_kind_arg(&cutoff_date_arg, provided_date) {
            Ok(x) => {x},
            Err(err) => { println!("Fatal error in fn elect_like_kind_treatment()."); process::exit(1) }
        };

        fn _elect_like_kind_arg(cutoff_date_arg: &Option<String>, provided_date: NaiveDate) -> Result<(bool, String), Box<Error>> {

            let mut input = String::new();
            let stdin = io::stdin();
            stdin.lock().read_line(&mut input)?;


            match input.trim().to_ascii_lowercase().as_str() {
                "y" | "ye" | "yes" | "" => {
                    println!("   Using like-kind treatment through {}.\n", provided_date);
                    Ok( (true, cutoff_date_arg.clone().unwrap()) )
                },
                "n" | "no" => { println!("   Proceeding without like-kind treatment.\n");
                    Ok( (false, "1-1-1".to_string()) )
                },
                "c" | "change" => {
                    println!("Please enter your desired like-kind exchange treatment cutoff date.");
                    println!("  You must use the format %y-%m-%d (e.g., 2017-12-31, 17-12-31, and 9-6-1 are all acceptable).\n");
                    let mut input = String::new();
                    let stdin = io::stdin();
                    stdin.lock().read_line(&mut input)?;
                    utils::trim_newline(&mut input);
                    let newly_chosen_date = NaiveDate::parse_from_str(&input, "%y-%m-%d")
                        .unwrap_or(NaiveDate::parse_from_str(&input, "%Y-%m-%d")
                        .expect("Date entered has an incorrect format. Program must abort."));
                    //  TODO: figure out how to make this fail gracefully and let the user input the date again
                    println!("   Using like-kind treatment through {}.\n", newly_chosen_date);
                    Ok( (true, input) )
                },
                _   => {
                    println!("Please respond with 'y', 'n', or 'c' (or 'yes' or 'no' or 'change').");
                    _elect_like_kind_arg(&cutoff_date_arg, provided_date)
                }
            }
        }

        return (election, date)

    } else {

        println!("\nContinue without like-kind exchange treatment? [Y/n] ");

        let (election, date) = match _no_elect_like_kind_arg() {
            Ok(x) => {x},
            Err(err) => { println!("Fatal error in like_kind selection. Perhaps incorrect date format."); process::exit(1) }
        };

        fn _no_elect_like_kind_arg() -> Result<(bool, String), Box<Error>> {

            let mut input = String::new();
            let stdin = io::stdin();
            stdin.lock().read_line(&mut input)?;

            match input.trim().to_ascii_lowercase().as_str() {
                "y" | "ye" | "yes" | "" => { println!("   Proceeding without like-kind treatment.\n");
                    Ok( (false, "1-1-1".to_string()) )
                },
                "n" | "no" => {
                    println!("Please enter your desired like-kind exchange treatment cutoff date.");
                    println!("  You must use the format %y-%m-%d (e.g., 2017-12-31, 17-12-31, and 9-6-1 are all acceptable).\n");
                    let mut input = String::new();
                    let stdin = io::stdin();
                    stdin.lock().read_line(&mut input)?;
                    utils::trim_newline(&mut input);

                    let newly_chosen_date = NaiveDate::parse_from_str(&input, "%y-%m-%d")
                        .unwrap_or(NaiveDate::parse_from_str(&input, "%Y-%m-%d")
                        .expect("Date entered has an incorrect format. Program must abort."));
                    //  TODO: figure out how to make this fail gracefully and let the user input the date again
                    println!("   Using like-kind treatment through {}.\n", newly_chosen_date);

                    Ok( (true, input) )
                },
                _   => { println!("Please respond with 'y' or 'n' (or 'yes' or 'no')."); _no_elect_like_kind_arg() }
            }
        }

        return (election, date)
    }
}

