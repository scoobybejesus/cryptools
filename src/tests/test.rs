// Copyright (c) 2017-2019, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools-rs/LEGAL.txt

use std::fs;
use std::collections::{HashMap};

use decimal::d128;

use crate::account::{Account};
use crate::transaction::{Transaction, ActionRecord};
use crate::decimal_utils::*;

pub fn run_tests(
    transactions_map: &HashMap<u32, Transaction>,
    action_records_map: &HashMap<u32, ActionRecord>,
    account_map: &HashMap<u16, Account>,
) {

    compare_movements_across_implementations(
        &transactions_map,
        &action_records_map,
        &account_map
    );

    do_mvmts_know_what_lot_they_are_in(&account_map);

    test_action_records_amts_vs_mvmt_amts(
        &transactions_map,
        &action_records_map,
        &account_map
    );

    test_quantize_from_incoming_multiple_lots_fn(d128!(20), d128!(200), d128!(50));
    test_quantize_from_incoming_multiple_lots_fn(d128!(1), d128!(6), d128!(1234567.1234567896));
    // test_dec_rounded("123456789.123456789");
    // test_dec_rounded("123456.123456");
    // test_dec_rounded("1234567891234.1234567891234");
    // test_dec_rounded_1e8("123456789.123456789");
    // test_dec_rounded_1e8("123456.123456");
    // test_dec_rounded_1e8("1234567891234.1234567891234");
    // test_dec_rounded_1e2("123456789.123456789");
    // test_dec_rounded_1e2("123456.123456");
    // test_dec_rounded_1e2("1234567891234.1234567891234");

}

fn compare_movements_across_implementations(
    transactions_map: &HashMap<u32, Transaction>,
    action_records_map: &HashMap<u32, ActionRecord>,
    account_map: &HashMap<u16, Account>,
) {
    //  THIS BIG ASS TEST CREATES TWO FILES TO COMPARE actionrecords.movements with ar.get_mvmts_in_ar()
    //  DELETE THIS WHEN movements FIELD OF ActionRecords IS DELETED

    let mut line: String = "".to_string();

    for tx_num in 1..=transactions_map.len() {
        let txn_num = tx_num as u32;
        line += &("Transaction ".to_string() + &txn_num.to_string() + &"\n".to_string());
        let txn = transactions_map.get(&(txn_num)).unwrap();
        for (idx, ar_num) in txn.action_record_idx_vec.iter().enumerate() {
            line += &("ActionRecord index: ".to_string() + &idx.to_string());
            let ar = action_records_map.get(&(ar_num)).unwrap();
            line += &(
                " with actionRecord key: ".to_string()
                + &ar_num.to_string()
                + " with amount: "
                + &ar.amount.to_string() + &"\n".to_string()
            );
            let mvmts = ar.get_mvmts_in_ar(&account_map, &transactions_map);
            let mut amts = d128!(0);
            for mvmt in mvmts {
                amts += mvmt.amount;
                line += &("Movement ".to_string() +
                                    &mvmt.amount.to_string() +
                                    &" on ".to_string() +
                                    &mvmt.date_as_string +
                                    &"\n".to_string());
            }
            line += &("Amount total: ".to_string() + &amts.to_string() + &"\n".to_string());
            if amts - ar.amount != d128!(0) {
                line += &("&*&*&*&*&*&*&*&*&*&*&*&*&*&*&*&*&*&*&*&*&*&*&*&*&*&*&*&".to_string());
                println!("Movement amounts via get_mvmts_in_ar() different from actionRecord.amount.  Aborting.");
                use std::process::exit; exit(1)
            };
        }
    }
    fs::write("/tmp/foo", &line).expect("Unable to write file");

    let mut line2: String = "".to_string();

    for tx_num in 1..=transactions_map.len() {
        let txn_num = tx_num as u32;
        line2 += &("Transaction ".to_string() + &txn_num.to_string() + &"\n".to_string());
        let txn = transactions_map.get(&(txn_num)).unwrap();
        for (idx, ar_num) in txn.action_record_idx_vec.iter().enumerate() {
            line2 += &("ActionRecord index: ".to_string() + &idx.to_string());
            let ar = action_records_map.get(&(ar_num)).unwrap();
            line2 += &(
                " with actionRecord key: ".to_string()
                + &ar_num.to_string()
                + " with amount: "
                + &ar.amount.to_string()
                + &"\n".to_string()
            );
            // let mvmts = ar.get_mvmts_in_ar(&account_map);
            let mut amts = d128!(0);
            for mvmt in ar.movements.borrow().iter() {
                amts += mvmt.amount;
                line2 += &("Movement ".to_string() +
                                    &mvmt.amount.to_string() +
                                    &" on ".to_string() +
                                    &mvmt.date_as_string +
                                    &"\n".to_string());
            }
            line2 += &("Amount total: ".to_string() + &amts.to_string() + &"\n".to_string());
            if amts - ar.amount != d128!(0) {
                line2 += &("&*&*&*&*&*&*&*&*&*&*&*&*&*&*&*&*&*&*&*&*&*&*&*&*&*&*&*&".to_string());
                println!("Movement amounts via ar.movements different from actionRecord.amount.  Aborting.");
                use std::process::exit; exit(1)
            };
        }
    }
    fs::write("/tmp/foo2", &line2).expect("Unable to write file");
}

fn do_mvmts_know_what_lot_they_are_in(account_map: &HashMap<u16, Account>,) {

    for (acct_num, acct) in account_map.iter() {
        for lot in acct.list_of_lots.borrow().iter() {
            for mvmt in lot.movements.borrow().iter() {
                if mvmt.lot_num != lot.lot_number {
                    println!("ERROR: For txn {} on {}, movement.lot_num != lot.lot_number.",
                        mvmt.transaction_key, mvmt.date);
                }
            }
        }
    }
}

pub fn test_action_records_amts_vs_mvmt_amts(
    transactions_map: &HashMap<u32, Transaction>,
    action_records_map: &HashMap<u32, ActionRecord>,
    account_map: &HashMap<u16, Account>,
) {

    let mut mvmt_amt_acct_lot: d128 = d128!(0);
    let mut mvmt_amt_ar: d128 = d128!(0);
    let mut ar_amts: d128 = d128!(0);

    for tx_num in 1..=transactions_map.len() {

        let txn_num = tx_num as u32;
        let txn = transactions_map.get(&(txn_num)).unwrap();

        for ar_num in &txn.action_record_idx_vec {

            let ar = action_records_map.get(&(ar_num)).unwrap();
            let mvmts = ar.get_mvmts_in_ar(&account_map, &transactions_map);
                for mvmt in mvmts {
                    mvmt_amt_ar += mvmt.amount
                }
        }
    }

    for acct_num in 1..=account_map.len() {

        let acct = account_map.get(&(acct_num as u16)).unwrap();

        for lot in acct.list_of_lots.borrow().iter() {
            for mvmt in lot.movements.borrow().iter() {
                mvmt_amt_acct_lot += mvmt.amount
            }
        }
    }

    for tx_num in 1..=transactions_map.len() {

        let txn_num = tx_num as u32;
        let txn = transactions_map.get(&(txn_num)).unwrap();

        for ar_num in &txn.action_record_idx_vec {
            let ar = action_records_map.get(&(ar_num)).unwrap();
            ar_amts += ar.amount
        }
    }

    println!("  \n\t\t\tActionRecord amounts pulled from iterating over transactions: {}
                \n\t\t\tMovement amounts pulled from actionRecords dynamically after: {}
                \n\t\t\tMovement amounts where movements are recorded directly in lots: {}\n",
                ar_amts,
                mvmt_amt_ar,
                mvmt_amt_acct_lot
    );
}

fn test_quantize_from_incoming_multiple_lots_fn (
    outgoing_mvmt_amt: d128,
    outgoing_ar_amt: d128,
    incoming_ar_amt: d128,
) {
    let rounded_example = d128::from(1).scaleb(d128::from(-8));
    //
    println!("Og mvmt amt: {:?}, Og ar amt: {:?}, Ic ar amt: {:?}", outgoing_mvmt_amt, outgoing_ar_amt, incoming_ar_amt);
    let ratio_of_outgoing_to_total_ar = outgoing_mvmt_amt / outgoing_ar_amt; //  Negative divided by negative is positive
    println!("ratio_of_outgoing: {:.20}", ratio_of_outgoing_to_total_ar);
    let tentative_incoming_amt = ratio_of_outgoing_to_total_ar * incoming_ar_amt;
    println!("tentative_inc_amt: {:.20}", tentative_incoming_amt);
    let corresponding_incoming_amt = tentative_incoming_amt.quantize(rounded_example);
    println!("corresponding_inc_amt: {}", corresponding_incoming_amt);
}

// Yields:
//     Og mvmt amt: 20, Og ar amt: 200, Ic ar amt: 50
//     ratio_of_outgoing: 0.1
//     tentative_inc_amt: 5.0
//     corresponding_inc_amt: 5.00000000
//     Og mvmt amt: 1, Og ar amt: 6, Ic ar amt: 1234567.1234567896
//     ratio_of_outgoing: 0.166666666666666666
//     tentative_inc_amt: 205761.1872427982666
//     corresponding_inc_amt: 205761.18724280

fn test_dec_rounded(random_float_string: &str) {
    let places_past_decimal = d128!(8);
    let amt = random_float_string.parse::<d128>().unwrap();
    let amt2 = round_d128_generalized(&amt, places_past_decimal);
    println!("Float into d128: {:?}; d128 rounded to {}: {:?}", amt, places_past_decimal, amt2);
    //  Results of this test suggest that quantize() is off by one.  round_d128_1e8() was adjusted accordingly.
}

fn test_dec_rounded_1e8(random_float_string: &str) {
    let amt = random_float_string.parse::<d128>().unwrap();
    let amt2 = round_d128_1e8(&amt);
    println!("Float into d128: {:?}; d128 rounded to 8 places: {:?}", amt, amt2);
    //  Results of this test suggest that quantize() is off by one.  round_d128_1e8() was adjusted accordingly.
}

fn test_dec_rounded_1e2(random_float_string: &str) {
    let amt = random_float_string.parse::<d128>().unwrap();
    let amt2 = round_d128_1e2(&amt);
    println!("String into d128: {:?}; d128 rounded to 2 places: {:?}", amt, amt2);
    //  Results of this test suggest that quantize() is off by one.  round_d128_1e8() was adjusted accordingly.
}
