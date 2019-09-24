// Copyright (c) 2017-2019, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools/blob/master/LEGAL.txt

use std::error::Error;
use std::process;
use std::fs::File;
use std::cell::{RefCell};
use std::collections::{HashMap};

use chrono::NaiveDate;
use decimal::d128;

use crate::transaction::{Transaction, ActionRecord};
use crate::account::{Account, RawAccount};
use crate::decimal_utils::{round_d128_1e8};


pub(crate) fn import_accounts(
    rdr: &mut csv::Reader<File>,
    raw_acct_map: &mut HashMap<u16, RawAccount>,
    acct_map: &mut HashMap<u16, Account>,
) -> Result<(), Box<dyn Error>> {

    let header1 = rdr.headers()?.clone();   //  account_num
    let mut header2: Option<csv::StringRecord> = None;  //  name
    let mut header3: Option<csv::StringRecord> = None;  //  ticker
    let header4: csv::StringRecord; //  is_margin

    // A StringRecord doesn't accept the same range indexing we need below, so we create our own
    let mut headerstrings: Vec<String> = Vec::with_capacity(header1.len());

    for element in header1.into_iter() {
        headerstrings.push(element.to_string())
    }

    // Account Creation loop.  We set hasheaders() to true above, so the first record here is the second row of the CSV
    for result in rdr.records() {
        //  This initial iteration through records will break after the 4th row, after accounts have been created
        let record = result?;
        if header2 == None {
            header2 = Some(record.clone());
            continue    //  After header2 is set, continue to next record
        }
        else if header3 == None {
            header3 = Some(record.clone());
            continue    //  After header3 is set, continue to next record
        }
        else {
            header4 = record.clone();
            // println!("Assigned last header, record: {:?}", record);

            let warn = "FATAL: Transactions will not import correctly if account numbers in the CSV import file aren't
ordered chronologically (i.e., beginning in column 4 - the 1st account column - the value should be 1.
The next column's value should be 2, then 3, etc, until the final account).";

            // We've got all our header rows.  It's now that we set up the accounts.
            println!("Attempting to create accounts...");

            let mut no_dup_acct_nums = HashMap::new();
            let length = &headerstrings.len();

            for num in headerstrings[3..*length].iter().enumerate() {
                let counter = no_dup_acct_nums.entry(num).or_insert(0);
                *counter += 1;
            }

            for acct_num in no_dup_acct_nums.keys() {
                assert_eq!(no_dup_acct_nums[acct_num], 1, "Found accounts with duplicate numbers during import.");
            }

            for (idx, item) in headerstrings[3..*length].iter().enumerate() {

                // println!("Headerstrings value: {:?}", item);
                let ind = idx+3; // Add three because the idx skips the first three 'key' columns
                let account_num = item.parse::<u16>()?;
                assert_eq!((idx + 1) as u16, account_num, "Found improper Account Number usage: {}", warn);

                let name:String = header2.clone().unwrap()[ind].trim().to_string();
                let ticker:String = header3.clone().unwrap()[ind].trim().to_string();   //  no .to_uppercase() b/c margin...
                let margin_string = &header4.clone()[ind];

                let is_margin:bool = match margin_string.trim().to_lowercase().as_str() {
                    "no" | "non" | "false" => false,
                    "yes" | "margin" | "true" => true,
                    _ => { println!("\n Couldn't parse margin value for acct {} {} \n",account_num, name); process::exit(1) }
                };

                let just_account: RawAccount = RawAccount {
                    account_num: account_num,
                    name: name,
                    ticker: ticker,
                    is_margin: is_margin,
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

pub(crate) fn import_transactions(
    rdr: &mut csv::Reader<File>,
    txns_map: &mut HashMap<u32, Transaction>,
    action_records: &mut HashMap<u32, ActionRecord>,
) -> Result<(), Box<dyn Error>> {

    let mut this_tx_number = 0;
    let mut this_ar_number = 0;
    let mut changed_action_records = 0;
    let mut changed_txn_num = Vec::new();

    println!("Attempting to create transactions...");

    for result in rdr.records() {

        //  rdr's cursor is at row 5, which is the first transaction row
        let record = result?;
        this_tx_number += 1;

        //  First, initialize metadata fields.
        let mut this_tx_date: &str = "";
        let mut this_proceeds: &str = "";
        let mut this_memo: &str = "";
        let mut this: String;
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
                this = field.replace(",", "");
                this_proceeds = this.as_str();
                proceeds_parsed = this_proceeds.parse::<f32>()?;
            }
            else if idx == 2 { this_memo = field; }

            //  Check for empty strings. If not empty, it's a value for an action_record.
            else if field != "" {
                this_ar_number += 1;
                let ind = idx;  //  starts at 3, which is the fourth field
                let acct_idx = ind - 2; //  acct_num and acct_key would be idx + 1, so subtract 2 from ind to get 1
                let account_key = acct_idx as u16;

                let amount_str = field.replace(",", "");
                let amount = amount_str.parse::<d128>().unwrap();
                let amount_rounded = round_d128_1e8(&amount);
                if amount != amount_rounded { changed_action_records += 1; changed_txn_num.push(this_tx_number); }

                let action_record = ActionRecord {
                    account_key: account_key,
                    amount: amount_rounded,
                    tx_key: this_tx_number,
                    self_ar_key: this_ar_number,
                    movements: RefCell::new([].to_vec()),
                };

                if amount > d128!(0.0) {
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

        match incoming_ar {
            Some(incoming_ar) => {
                let x = incoming_ar_num.unwrap();
                action_records.insert(x, incoming_ar);
            },
            None => {}
        }

        match outgoing_ar {
            Some(outgoing_ar) => {
                let y = outgoing_ar_num.unwrap();
                action_records.insert(y, outgoing_ar);
            },
            None => {}
        }

        let tx_date = NaiveDate::parse_from_str(this_tx_date, "%m/%d/%y")
            .unwrap_or(NaiveDate::parse_from_str(this_tx_date, "%m/%d/%Y")
            .expect("%m/%d/%y (or %Y) format required for ledger import")
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
        println!("  Changed actionrecord amounts: {}. Changed txn numbers: {:?}.", changed_action_records, changed_txn_num);
    }

    Ok(())
}
