// Copyright (c) 2017-2023, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools/blob/master/LEGAL.txt

use std::error::Error;
use std::process;
use std::fs::File;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::transaction::{Transaction, ActionRecord};
use crate::account::{Account, RawAccount};
use crate::decimal_utils::round_d128_1e8;


pub fn import_from_csv(
    import_file_path: PathBuf,
    iso_date_style: bool,
    separator: &String,
    raw_acct_map: &mut HashMap<u16, RawAccount>,
    acct_map: &mut HashMap<u16, Account>,
    action_records: &mut HashMap<u32, ActionRecord>,
    transactions_map: &mut HashMap<u32, Transaction>,
) -> Result<(), Box<dyn Error>> {

    let file = match File::open(import_file_path) {
        Ok(x) => {
            // println!("\nCSV ledger file opened successfully.\n");
            x
        },
        Err(e) => {
            println!("Invalid import_file_path");
            eprintln!("System error: {}", e);
            std::process::exit(1);
        }
    };

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);

    import_accounts(&mut rdr, raw_acct_map, acct_map)?;

    import_transactions(
        &mut rdr,
        iso_date_style,
        &separator,
        action_records,
        transactions_map,
    )?;

    Ok(())
}

fn import_accounts(
    rdr: &mut csv::Reader<File>,
    raw_acct_map: &mut HashMap<u16, RawAccount>,
    acct_map: &mut HashMap<u16, Account>,
) -> Result<(), Box<dyn Error>> {

    let header1 = rdr.headers()?.clone();   //  account_num
    let mut header2: csv::StringRecord = csv::StringRecord::new();  //  name
    let mut header3: csv::StringRecord = csv::StringRecord::new();  //  ticker
    let header4: csv::StringRecord; //  is_margin

    // Account Creation loop.  With rdr.has_headers() set to true above, the first record here is the second row of the CSV
    for result in rdr.records() {
        //  This initial iteration through records will break after the 4th row, after accounts have been created
        let record = result?;
        if header2.len() == 0 {
            header2 = record.clone();
            continue    //  After header2 is set, continue to next record
        }
        else if header3.len() == 0 {
            header3 = record.clone();
            continue    //  After header3 is set, continue to next record
        }
        else {
            header4 = record.clone();
            // println!("Assigned last header, record: {:?}", record);

            // A StringRecord doesn't accept the same range indexing needed below, so a Vec of Strings will be used
            let headerstrings: Vec<String> = header1.into_iter().map(|field| field.to_string()).collect();

            let acct_num_warn = "Transactions will not import correctly if account numbers in the CSV import file aren't
ordered chronologically (i.e., beginning in column 4 - the 1st account column - the value should be 1.
The next column's value should be 2, then 3, etc, until the final account).";

            // Header row variables have been set.  It's now time to set up the accounts.
            println!("\nCreating accounts...");

            let length = &headerstrings.len();

            for (idx, field) in headerstrings[3..*length].iter().enumerate() {

                // Parse account numbers.
                let account_num = field.trim().parse::<u16>().expect(&format!("Header row account number should parse into u16: {}", field));
                // For now, their columns aren't remembered.  Instead, they must have a particular index. 0th idx is the 1st account, and so on.
                if account_num != ((idx + 1) as u16) {
                    println!("FATAL: CSV Import: {}", acct_num_warn);
                    std::process::exit(1);
                }

                let ind = idx+3; // Add three because the idx skips the first three 'key' columns
                let name:String = header2[ind].trim().to_string();
                let ticker:String = header3[ind].trim().to_string();   //  no .to_uppercase() b/c margin...
                let margin_string = &header4.clone()[ind];

                let is_margin:bool = match margin_string.to_lowercase().trim() {
                    "no" | "non" | "false" => false,
                    "yes" | "margin" | "true" => true,
                    _ => {
                        println!("\n FATAL: CSV Import: Couldn't parse margin value for account {} {} \n",account_num, name);
                        process::exit(1)
                    }
                };

                let just_account: RawAccount = RawAccount {
                    account_num,
                    name,
                    ticker,
                    is_margin,
                };

                raw_acct_map.insert(account_num, just_account);

                let account: Account = Account {
                    raw_key: account_num,
                    list_of_lots: RefCell::new([].to_vec())
                };

                acct_map.insert(account_num, account);
            }
            break    //  This `break` exits this scope so `accounts` can be accessed in `import_transactions`. The rdr stays put.
        }
    };
    Ok(())
}

fn import_transactions(
    rdr: &mut csv::Reader<File>,
    iso_date_style: bool,
    separator: &String,
    action_records: &mut HashMap<u32, ActionRecord>,
    txns_map: &mut HashMap<u32, Transaction>,
) -> Result<(), Box<dyn Error>> {

    let mut this_tx_number = 0;
    let mut this_ar_number = 0;
    let mut changed_action_records = 0;
    let mut changed_txn_num = Vec::new();

    println!("Creating transactions...");

    for result in rdr.records() {

        //  rdr's cursor is at row 5, which is the first transaction row
        let record = result?;
        this_tx_number += 1;

        //  First, initialize metadata fields.
        let mut this_tx_date: &str = "";
        let mut this_proceeds: &str;
        let mut this_memo: &str = "";
        let mut proceeds_parsed = 0f32;

        //  Next, create action_records.
        let mut action_records_map_keys_vec: Vec<u32> = Vec::with_capacity(2);
        let mut outgoing_ar: Option<ActionRecord> = None;
        let mut incoming_ar: Option<ActionRecord> = None;
        let mut outgoing_ar_num: Option<u32> = None;
        let mut incoming_ar_num: Option<u32> = None;

        for (idx, field) in record.iter().enumerate() {

            //  Set metadata fields on first three fields.
            if idx == 0 { this_tx_date = field; }
            else if idx == 1 {
                let no_comma_string = field.replace(",", "");
                proceeds_parsed = no_comma_string.parse::<f32>()?;
            }

            else if idx == 2 { this_memo = field; }

            //  Check for empty strings. If not empty, it's a value for an action_record.
            else if field != "" {
                this_ar_number += 1;
                let ind = idx;  //  starts at 3, which is the fourth field
                let acct_idx = ind - 2; //  acct_num and acct_key would be idx + 1, so subtract 2 from ind to get 1
                let account_key = acct_idx as u16;

                let amount_str = field.replace(",", "");
                let amount = match amount_str.parse::<Decimal>() {
                    Ok(x) => x,
                    Err(e) => {
                        println!("FATAL: Couldn't convert amount to d128 for transaction:\n{:#?}", record);
                        println!("Error: {}", e);
                        std::process::exit(1);}
                };

                // When parsing to a d128, it won't error; rather it'll return a NaN. It must now check for NaN,
                // and, if found, attempt to sanitize.  These checks will convert accounting/comma format to the expected
                // format by removing parentheses from negatives and adding a minus sign in the front. It will also
                // attempt to remove empty spaces and currency symbols or designations (e.g. $ or USD).
                // if amount.is_none() {
                //     let b = sanitize_string_for_d128_parsing_basic(field).parse::<Decimal>().unwrap();
                //     amount = b;
                // };
                // if amount.is_none() {
                //     let c = sanitize_string_for_d128_parsing_full(field).parse::<Decimal>().unwrap();
                //     amount = c;
                // };
                // if amount.is_none() {
                //     println!("FATAL: Couldn't convert amount to d128 for transaction:\n{:#?}", record);
                //     std::process::exit(1);
                // }

                let amount_rounded = round_d128_1e8(&amount);
                if amount != amount_rounded { changed_action_records += 1; changed_txn_num.push(this_tx_number); }

                let action_record = ActionRecord {
                    account_key,
                    amount: amount_rounded,
                    tx_key: this_tx_number,
                    self_ar_key: this_ar_number,
                    movements: RefCell::new([].to_vec()),
                };

                if amount > dec!(0.0) {
                    incoming_ar = Some(action_record);
                    incoming_ar_num = Some(this_ar_number);
                    action_records_map_keys_vec.push(incoming_ar_num.unwrap())
                } else {
                    outgoing_ar = Some(action_record);
                    outgoing_ar_num = Some(this_ar_number);
                    action_records_map_keys_vec.insert(0, outgoing_ar_num.unwrap())
                };
            }
        }

        // Note: the rust Trait implementation of FromStr for f32 is capable of parsing:
            // '3.14'
            // '-3.14'
            // '2.5E10', or equivalently, '2.5e10'
            // '2.5E-10'
            // '5.'
            // '.5', or, equivalently, '0.5'
            // 'inf', '-inf', 'NaN'
        // Notable observations from the list:
            // (a) scientific notation is accepted
            // (b) accounting format (numbers in parens representing negative numbers) is not explicitly accepted
        // Additionally notable:
            // (a) the decimal separator must be a period
            // (b) there can be no commas
            // (c) there can be no currency info ($120 or 120USD, etc. will fail to parse)
        // In summary, it appears to only allow: (i) numeric chars, (ii) a period, and/or (iii) a minus sign
        //
        // The Decimal::d128 implementation of FromStr calls into a C library, and that lib hasn't
        // been reviewed (by me), but it is thought/hoped to follow similar parsing conventions,
        // though there's no guarantee.  Nevertheless, the above notes *appear* to hold true for d128.
        // fn sanitize_string_for_d128_parsing_basic(field: &str) -> String {

        //     // First, remove commas.
        //     let no_comma_string = field.replace(",", "");
        //     let almost_done = no_comma_string.replace(" ", "");

        //     // Next, if ASCII (better be), check for accounting formatting
        //     if almost_done.is_ascii() {
        //         if almost_done.as_bytes()[0] == "(".as_bytes()[0] {
        //             let half_fixed = almost_done.replace("(", "-");
        //             let negative_with_minus = half_fixed.replace(")", "");
        //             return negative_with_minus
        //         }
        //     }
        //     almost_done
        // }

        // fn sanitize_string_for_d128_parsing_full(field: &str) -> String {

        //     let mut near_done = "".to_string();
        //     // First, remove commas.
        //     let no_comma_string = field.replace(",", "");
        //     let almost_done = no_comma_string.replace(" ", "");

        //     // Next, if ASCII (better be), check for accounting formating
        //     if almost_done.is_ascii() {
        //         if almost_done.as_bytes()[0] == "(".as_bytes()[0] {
        //             let half_fixed = almost_done.replace("(", "-");
        //             let negative_with_minus = half_fixed.replace(")", "");
        //             near_done = negative_with_minus;
        //         } else {
        //             near_done = almost_done;
        //         }
        //     } else {
        //         near_done = almost_done;
        //     }

        //     // Strip non-numeric and non-period characters
        //     let all_done: String = near_done.chars()
        //         .filter(|x|
        //             x.is_numeric() |
        //             (x == &(".".as_bytes()[0] as char)) |
        //             (x == &("-".as_bytes()[0] as char)))
        //             .collect();
        //     all_done
        // }

        if let Some(incoming_ar) = incoming_ar {
            let x = incoming_ar_num.unwrap();
            action_records.insert(x, incoming_ar);
        }

        if let Some(outgoing_ar) = outgoing_ar {
            let y = outgoing_ar_num.unwrap();
            action_records.insert(y, outgoing_ar);
        }

        let format_yy: String;
        let format_yyyy: String;

        if iso_date_style {
            format_yyyy = "%Y".to_owned() + separator + "%m" + separator + "%d";
            format_yy = "%y".to_owned() + separator + "%m" + separator + "%d";
        } else {
            format_yyyy = "%m".to_owned() + separator + "%d" + separator + "%Y";
            format_yy = "%m".to_owned() + separator + "%d" + separator + "%y";
        }

        let tx_date = NaiveDate::parse_from_str(this_tx_date, &format_yy)
            .unwrap_or_else(|_| NaiveDate::parse_from_str(this_tx_date, &format_yyyy)
            .expect("
FATAL: Transaction date parsing failed. You must tell the program the format of the date in your CSV Input File. The date separator \
is expected to be a hyphen. The dating format is expected to be \"American\" (%m-%d-%y), not ISO 8601 (%y-%m-%d). You may set different \
date format options via command line flag, environment variable or .env file. Perhaps first run with `--help` or see `.env.example.`\n")
        );

        let transaction = Transaction {
            tx_number: this_tx_number,
            date_as_string: this_tx_date.to_string(),
            date: tx_date,
            user_memo: this_memo.to_string(),
            proceeds: proceeds_parsed,
            action_record_idx_vec: action_records_map_keys_vec,
        };

        txns_map.insert(this_tx_number, transaction);
    };

    if changed_action_records > 0 {
        println!("  Changed actionrecord amounts due to rounding precision: {}. Changed txn numbers: {:?}.", changed_action_records, changed_txn_num);
    }

    Ok(())
}
