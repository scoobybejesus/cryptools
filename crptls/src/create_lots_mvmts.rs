// Copyright (c) 2017-2020, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools/blob/master/LEGAL.txt

use std::rc::Rc;
use std::cell::{RefCell, Ref, Cell};
use std::collections::HashMap;
use std::error::Error;

use decimal::d128;

use crate::core_functions::ImportProcessParameters;
use crate::transaction::{Transaction, ActionRecord, TxType, Polarity, TxHasMargin};
use crate::account::{Account, RawAccount, Lot, Movement};
use crate::costing_method::InventoryCostingMethod;
use crate::decimal_utils::round_d128_1e8;

/// This is probably the most important function in the whole program.  Based on the data in the CSV Input File,
/// the `account`s and `transaction`s will be created.  Once the `account`s and `transaction`s have been created, both
/// are passed to this function for `lot` processing.  The `lot` processing rules can be deduced from reading the code,
/// though that implies there are no mistakes (i.e., a mistake in the code would mean that the intent of the code cannot
/// necessarily be deduced).  To remove any doubt, the logic below will be made [hopefully] clear by documentation.
///
/// The first thing to bear in mind is that this function should not be thought of as generalizable to an interactive
/// program; rather, the only data this program requires is that which is included in the CSV Input File, and accordingly
/// the program will iterate through that data deterministically to produce the results that it produces.  On top of that,
/// the program is assumed to produce correct results based on a correct CSV Input File.  Accordingly, this code is designed
/// to fail loud and fast if it encounters unexpected things.  Logging, particularly in this function, would be wise.
///
/// Second, the goal of this function is to be able to properly record all `lot`s and `lot` `movements` in a single
/// iteration.  This means there is a strict order of operations required in some parts.  For example, in the case of a
/// like-kind exchange `transaction` in which the basis dates and amounts of the outgoing `action record` must be transferred
/// to the basis dates and amounts of the corresponding incoming `action record`, the transaction's outgoing `action record`
/// therefore must be processed before that `transction`'s incoming `action record`.  For that reason, a `transaction`'s
/// vector of `action record` indices is ordered with outgoing `action record`s always being first.
///
/// Third, `lot`s and `movement`s are created sequentially and kept in that order.  Accordingly, an outgoing `movement` will
/// reduce the last `lot` first.  If there is not enough in the last `lot`, the remainder will then be applied to the `lot`
/// before that, and the `lot` before that, and so on (recursively), until the amount in the `action record` has been
/// fully recorded as `lot` `movement`s.  This is known as last in, first out (LIFO).  Please refer to the `InventoryCostingMethod`
/// enum for the available choices if LIFO is not desirable. If a different `inventory costing method` is chosen,
/// the vector of indices is re-ordered to accomodate the paradigm that the `action record` amount will always be applied
/// to the last `lot` (i.e., if e.g. FIFO is chosen, then the index for the last `lot` will be 0 instead of length - 1.).
/// See below `vec_of_ordered_index_values`.
///
/// Fourth, this function does not contemplate any income/expense/gain/loss at all.  It is solely an exercise in determining
/// and solidifying how to split (if needed) the amount in each `action record` into `movement`s that post to the appropriate
/// `lot`s.  Conceptually, each `account` has a list of `lot`s, and each `lot` has a list of `movement`s.
pub(crate) fn create_lots_and_movements(
    settings: &ImportProcessParameters,
    raw_acct_map: &HashMap<u16, RawAccount>,
    acct_map: &HashMap<u16, Account>,
    ar_map: &HashMap<u32, ActionRecord>,
    txns_map: HashMap<u32, Transaction>,
    // lot_map: &HashMap<(RawAccount, u32), Lot>,
) -> Result<HashMap<u32,Transaction>, Box<dyn Error>> {

    let chosen_home_currency = &settings.home_currency;
    let chosen_costing_method = &settings.costing_method;
    let enable_lk_treatment = settings.lk_treatment_enabled;
    let like_kind_cutoff_date = settings.lk_cutoff_date;
    let lk_basis_date_preserved = settings.lk_basis_date_preserved;

    // This is set automatically based on how like-kind `exchange` `transaction`s work, but it could be left to user choice, in theory.
    let multiple_incoming_mvmts_per_ar_due_to_lk = lk_basis_date_preserved;

    let length = txns_map.len();

    // Transactions are stored in a HashMap, and they are ordered sequentially starting at 1, so we iterate through
    // that range and use the corresponding `num` to get each transaction.
    for num in 1..=length {

        let txn_num = num as u32;
        let txn = txns_map.get(&(txn_num)).expect("Couldn't get txn. Tx num invalid?");

        // The first type of transaction we consider are those where both `action record`s have an `account` that
        // is a margin `account`.  If so, it is an `exchange` `transaction`.  `Exchange` `transaction`s for margin
        // `account`s don't create a new lot for every increase.  Rather, it keeps one lot per "close," which is
        // to say that pair of margin `account` `lot`s will be used until a profit or loss is realized as a result
        // of zeroing out both the margin `account`s by both closing the margin position AND making a transfer
        // between the margin quote `account` and the corresponding spot `account` such that both margin `account`s
        // now have a zero balance.
        if txn.marginness(&ar_map, &raw_acct_map, &acct_map) == TxHasMargin::TwoARs {
            assert_eq!(txn.transaction_type(&ar_map, &raw_acct_map, &acct_map)?, TxType::Exchange);
            assert_eq!(txn.action_record_idx_vec.len(), 2);

            let the_raw_pair_keys = txn.get_base_and_quote_raw_acct_keys(&ar_map, &raw_acct_map, &acct_map)?;
            let base_acct = acct_map.get(&the_raw_pair_keys.0).expect("Couldn't get acct. Raw pair keys invalid?");
            let quote_acct = acct_map.get(&the_raw_pair_keys.1).expect("Couldn't get acct. Raw pair keys invalid?");

            // This seems trivial, but there can be a series of buys and sells within a margin trade before the
            // trade is closed for a profit or loss, so this ensures we know which `action record` is which.
            let (base_ar_idx, quote_ar_idx) = get_base_and_quote_ar_idxs(
                the_raw_pair_keys,
                &txn,
                &ar_map,
                &raw_acct_map,
                &acct_map
            );

            // Unlike all logic following this `TxHasMargin::TwoARs` section, both `action record`s are handled at once.
            let base_ar = ar_map.get(&base_ar_idx).unwrap();
            let quote_ar = ar_map.get(&quote_ar_idx).unwrap();

            let mut base_acct_lot_list = base_acct.list_of_lots.borrow_mut();
            let mut quote_acct_lot_list = quote_acct.list_of_lots.borrow_mut();

            // The number of `lot`s between the quote and base `account`s should always be equal, and there is
            // an error in the code if they are not, so it is meant to panic if so.
            let base_number_of_lots = base_acct_lot_list.len() as u32;
            let quote_number_of_lots = quote_acct_lot_list.len() as u32;
            assert_eq!(base_number_of_lots, quote_number_of_lots, "");

            // The value is set just below.  We use this to determine whether to create a new `lot` for each `account`.
            let acct_balances_are_zero: bool;

            // Though checking both is redundant, we keep the redundancy for code clarity sake.  If true, the implication
            // is that there has already been activity in the margin `account`s in this `transaction`.
            if !base_acct_lot_list.is_empty() && !quote_acct_lot_list.is_empty() {
                // Since we know there has been activity, we set the bool variable above according to whether the `account`
                // balances are both zero.
                let base_balance_is_zero = base_acct_lot_list.last().unwrap().get_sum_of_amts_in_lot() == d128!(0);
                let quote_balance_is_zero = quote_acct_lot_list.last().unwrap().get_sum_of_amts_in_lot() == d128!(0);
                if base_balance_is_zero && quote_balance_is_zero {
                    acct_balances_are_zero = true
                } else {
                    acct_balances_are_zero = false
                }

            // If there has supposedly been no activity in the two margin `account`s in this `transaction`, we double
            // check that there are no `lot`s associated with either `account`, and then set the bool variable accordingly.
            } else {
                assert_eq!(true, base_acct_lot_list.is_empty(),
                    "One margin account's list_of_lots is empty, but its pair's isn't.");
                assert_eq!(true, quote_acct_lot_list.is_empty(),
                    "One margin account's list_of_lots is empty, but its pair's isn't.");
                acct_balances_are_zero = true
            }

            // The `lot` for each `account` is allocated here, and their values are set within other scopes underneath
            let base_lot: Rc<Lot>;
            let quote_lot: Rc<Lot>;

            // If both `account`s have zero balances, new `lot`s are created.  The variables created above will take
            // the assignment.
            if acct_balances_are_zero {
                base_lot = Rc::new(
                    Lot {
                        date_as_string: txn.date_as_string.clone(),
                        date_of_first_mvmt_in_lot: txn.date,
                        date_for_basis_purposes: txn.date,
                        lot_number: base_number_of_lots + 1,
                        account_key: the_raw_pair_keys.0,
                        movements: RefCell::new([].to_vec()),
                    }
                );
                quote_lot = Rc::new(
                    Lot {
                        date_as_string: txn.date_as_string.clone(),
                        date_of_first_mvmt_in_lot: txn.date,
                        date_for_basis_purposes: txn.date,
                        lot_number: quote_number_of_lots + 1,
                        account_key: the_raw_pair_keys.1,
                        movements: RefCell::new([].to_vec()),
                    }
                );

            // If at least one `account` has a balance, then each must have at least one `lot`, and those are the `lot`s
            // that will be assigned to the variables above.  If a `lot` can't be found, the data is wrong or the code is
            // wrong, and the program should panic.
            } else {
                base_lot = base_acct_lot_list.last().expect("Couldn't get lot. Base acct lot list empty?").clone();
                quote_lot = quote_acct_lot_list.last().expect("Couldn't get lot. Quote acct lot list empty?").clone();
            }

            // Now that each of the `lot`s is chosen, the `movement`s can be created (which contain the `lot` number)
            // and pushed onto their respective `lot`s.
            let base_mvmt = Movement {
                amount: base_ar.amount,
                date_as_string: txn.date_as_string.clone(),
                date: txn.date,
                transaction_key: txn_num,
                action_record_key: base_ar_idx,
                cost_basis: Cell::new(d128!(0.0)),
                ratio_of_amt_to_incoming_mvmts_in_a_r: d128!(1.0),
                ratio_of_amt_to_outgoing_mvmts_in_a_r: Cell::new(d128!(1.0)),
                lot_num: base_lot.lot_number,
                proceeds: Cell::new(d128!(0.0)),
                proceeds_lk: Cell::new(d128!(0.0)),
                cost_basis_lk: Cell::new(d128!(0.0)),
            };
            let raw_base_acct = raw_acct_map.get(&base_acct.raw_key).unwrap();
            wrap_mvmt_and_push(
                base_mvmt,
                &base_ar,
                &base_lot,
                &chosen_home_currency,
                &raw_base_acct,
            );

            let quote_mvmt = Movement {
                amount: quote_ar.amount,
                date_as_string: txn.date_as_string.clone(),
                date: txn.date,
                transaction_key: txn_num,
                action_record_key: quote_ar_idx,
                cost_basis: Cell::new(d128!(0.0)),
                ratio_of_amt_to_incoming_mvmts_in_a_r: d128!(1.0),
                ratio_of_amt_to_outgoing_mvmts_in_a_r: Cell::new(d128!(1.0)),
                lot_num: quote_lot.lot_number,
                proceeds: Cell::new(d128!(0.0)),
                proceeds_lk: Cell::new(d128!(0.0)),
                cost_basis_lk: Cell::new(d128!(0.0)),
            };
            let raw_quote_acct = raw_acct_map.get(&quote_acct.raw_key).unwrap();
            wrap_mvmt_and_push(
                quote_mvmt,
                &quote_ar,
                &quote_lot,
                &chosen_home_currency,
                &raw_quote_acct,
            );

            // Self-explanatory.  If new `lot`s were created, those `lot`s need to be pushed onto the `account`s.
            if acct_balances_are_zero {
                base_acct_lot_list.push(base_lot);
                quote_acct_lot_list.push(quote_lot);
            }

            // Once the `movement`s have been created and pushed to the appropriate `lot` (and the `lot` pushed to the appropriate
            // `account` if need be), then the transaction has been processed, and it can move onto the next.
            continue

        // If this isn't a margin `exchange` `transaction`, then the lot rules are different, and it continues below.
        } else {
            // Unlike all logic above, in the `TxHasMargin::TwoARs` section, each `action record` is handled one at a time.
            for ar_num in txn.action_record_idx_vec.iter() {
                let ar = ar_map.get(ar_num).unwrap();

                let acct = acct_map.get(&ar.account_key).unwrap();
                let raw_acct = raw_acct_map.get(&acct.raw_key).unwrap();
                let length_of_list_of_lots = acct.list_of_lots.borrow().len();

                // Each home currency `account` contains a single `lot`.  There is no restriction on its balance, unlike for
                // crypto `account`s which must always have a non-negative balance (except for margin `account`s, where one `account`
                // must necessarily go negative as the other goes postive).
                if raw_acct.is_home_currency(&chosen_home_currency) {
                    let lot;
                    let new_lot_created;

                    // If there is no `lot`, create a new one.  If there is one, use it.
                    if length_of_list_of_lots == 0 {
                        lot = Rc::new(
                            Lot {
                                date_as_string: txn.date_as_string.clone(),
                                date_of_first_mvmt_in_lot: txn.date,
                                date_for_basis_purposes: txn.date,
                                lot_number: 1,
                                account_key: acct.raw_key,
                                movements: RefCell::new([].to_vec()),
                            }
                        );
                        new_lot_created = true;
                    }
                    else {
                        assert_eq!(1, length_of_list_of_lots); //  Only true for home currency
                        lot = acct.list_of_lots.borrow_mut()[0 as usize].clone();
                        new_lot_created = false;
                    }

                    // Then create the movement and push it.
                    let whole_mvmt = Movement {
                        amount: ar.amount,
                        date_as_string: txn.date_as_string.clone(),
                        date: txn.date,
                        transaction_key: txn_num,
                        action_record_key: *ar_num,
                        cost_basis: Cell::new(d128!(0.0)),
                        ratio_of_amt_to_incoming_mvmts_in_a_r: d128!(1.0),
                        ratio_of_amt_to_outgoing_mvmts_in_a_r: Cell::new(d128!(1.0)),
                        lot_num: lot.lot_number,
                        proceeds: Cell::new(d128!(0.0)),
                        proceeds_lk: Cell::new(d128!(0.0)),
                        cost_basis_lk: Cell::new(d128!(0.0)),
                    };
                    wrap_mvmt_and_push(
                        whole_mvmt,
                        &ar,
                        &lot,
                        &chosen_home_currency,
                        &raw_acct,
                    );

                    // If there is a new `lot`, push it onto the `account`
                    if new_lot_created { acct.list_of_lots.borrow_mut().push(lot); }

                    // Whether incoming or outgoing, the home currency `action record` is now recorded correctly, and then
                    // onto the next `action record` or `transaction`.
                    continue
                }
                // Below here, every `action record`'s `account` is not home currency, so the program must know whether
                // `action record` is incoming/outgoing and whether the `transaction` `TxType` is `Exchange`/`ToSelf`/`Flow`.
                let polarity = ar.direction();
                let tx_type = txn.transaction_type(&ar_map, &raw_acct_map, &acct_map)?;

                // The `action record` handling is different depending on whether it's incoming or outgoing.
                match polarity {
                    Polarity::Outgoing => {
                        // println!("Txn: {}, outgoing {:?}-type of {} {}",
                            // txn.tx_number, txn.transaction_type(), ar.amount, acct.ticker);
                        //
                        // For an outgoing `action record` with a margin `account`, it can be deduced that there is a corresponding
                        // incoming `action record` with a non-margin `account.`  This setup (two `action record`s where one's `account`
                        // is home currency and the other's isn't) is referred to in this context as a dual-`action record` `flow`
                        // `transaction`. In this case (with an outgoing `action record` with the margin `account`), it is a margin
                        // profit `transaction` since the corresponding incoming `action record` increases a non-margin `account` balance.
                        //
                        // In order to withdraw margin profits, the margin base `account` must have a zero balance, and the margin quote
                        // `account` must have a positive balance. We know, therefore, that the `account` of this `action record` is the
                        // quote `account`, and the `lot` treatment is simple. A single `movement` posts to the active `lot` in this
                        // `account`, presumably (but not definitely) zeroing it out.
                        if raw_acct.is_margin {
                            let this_acct = acct_map.get(&ar.account_key).unwrap();
                            let lot = this_acct.list_of_lots.borrow().last()
                                .expect("Couldn't get lot. Acct lot list empty?").clone();
                            let whole_mvmt = Movement {
                                amount: ar.amount,
                                date_as_string: txn.date_as_string.clone(),
                                date: txn.date,
                                transaction_key: txn_num,
                                action_record_key: *ar_num,
                                cost_basis: Cell::new(d128!(0.0)),
                                ratio_of_amt_to_incoming_mvmts_in_a_r: d128!(1.0),
                                ratio_of_amt_to_outgoing_mvmts_in_a_r: Cell::new(d128!(1.0)),
                                lot_num: lot.lot_number,
                                proceeds: Cell::new(d128!(0.0)),
                                proceeds_lk: Cell::new(d128!(0.0)),
                                cost_basis_lk: Cell::new(d128!(0.0)),
                            };
                            wrap_mvmt_and_push(
                                whole_mvmt,
                                &ar,
                                &lot,
                                &chosen_home_currency,
                                &raw_acct,
                            );
                            continue

                        // For an outgoing `action record` with a non-margin `account`, this is where it is determined how to split
                        // the amount (if needed) into `movements` that "fit into" `lot`s.
                        } else {

                            if acct.list_of_lots.borrow().len() == 0 {
                                println!("FATAL: There are zero lots to spend from in transaction:\n{:#?}",txn);
                                std::process::exit(1);
                            }

                            let list_of_lots_to_use = acct.list_of_lots.clone();

                            //  The following returns a Vec to be iterated from beginning to end. It provides the index for the desired `lot`.
                            let vec_of_ordered_index_values = match chosen_costing_method {
                                InventoryCostingMethod::LIFObyLotCreationDate => {
                                    get_lifo_by_creation_date(&list_of_lots_to_use.borrow())}
                                InventoryCostingMethod::LIFObyLotBasisDate => {
                                    get_lifo_by_lot_basis_date(&list_of_lots_to_use.borrow())}
                                InventoryCostingMethod::FIFObyLotCreationDate => {
                                    get_fifo_by_creation_date(&list_of_lots_to_use.borrow())}
                                InventoryCostingMethod::FIFObyLotBasisDate => {
                                    get_fifo_by_lot_basis_date(&list_of_lots_to_use.borrow())}
                            };

                            assert_eq!(vec_of_ordered_index_values.len(), list_of_lots_to_use.borrow().len());

                            fn get_lifo_by_creation_date(list_of_lots: &Ref<Vec<Rc<Lot>>>) -> Vec<usize> {
                                let mut vec_of_indexes = [].to_vec(); // TODO: Add with_capacity()
                                for (idx, _lot) in list_of_lots.iter().enumerate() {
                                    vec_of_indexes.insert(0, idx)
                                }
                                vec_of_indexes
                            }

                            #[allow(suspicious_double_ref_op)]
                            fn get_lifo_by_lot_basis_date(list_of_lots: &Ref<Vec<Rc<Lot>>>) -> Vec<usize> {
                                let mut reordered_vec = list_of_lots.clone().to_vec();
                                let length = reordered_vec.len();
                                for _ in 0..length {
                                    for j in 0..length-1 {
                                        if reordered_vec[j].date_for_basis_purposes > reordered_vec[j+1].date_for_basis_purposes {
                                            reordered_vec.swap(j, j+1)
                                        }
                                    }
                                }
                                let mut vec_of_indexes = [].to_vec();
                                for (idx, _lot) in reordered_vec.iter().enumerate() {
                                    vec_of_indexes.insert(0, idx)
                                }
                                vec_of_indexes
                            }

                            fn get_fifo_by_creation_date(list_of_lots: &Ref<Vec<Rc<Lot>>>) -> Vec<usize> {
                                let mut vec_of_indexes = [].to_vec();
                                for (idx, _lot) in list_of_lots.iter().enumerate() {
                                    vec_of_indexes.push(idx)
                                }
                                vec_of_indexes
                            }

                            #[allow(suspicious_double_ref_op)]
                            fn get_fifo_by_lot_basis_date(list_of_lots: &Ref<Vec<Rc<Lot>>>) -> Vec<usize> {
                                let mut reordered_vec = list_of_lots.clone().to_vec();
                                let length = reordered_vec.len();
                                for _ in 0..length {
                                    for j in 0..length-1 {
                                        if reordered_vec[j].date_for_basis_purposes > reordered_vec[j+1].date_for_basis_purposes {
                                            reordered_vec.swap(j, j+1)
                                        }
                                    }
                                }
                                let mut vec_of_indexes = [].to_vec();
                                for (idx, _lot) in reordered_vec.iter().enumerate() {
                                    vec_of_indexes.push(idx)
                                }
                                vec_of_indexes
                            }

                            // TODO: Consider whether a for-loop can track the index more cleanly
                            // Now that the index values of each `lot` are in the appropriate order, the starting point (index 0)
                            // and the starting lot_index can be chosen in preparation for the recursive `fit_into_lots` function.
                            // If the tentative `movement` must be reduced to fit into the tentative `lot`, a revised `movement` will be created
                            // using an amount that will be reduced to the exact amount to fit into the `lot`.  After the revised `movement`
                            // is pushed to the `lot`, the index position will be incremented to provide a new `lot_index`, a new tentative
                            // `lot` will be chosen, and the remainder of the amount will be used in a new tentative `movement`, and so on
                            // until the entire `action record` amount has been put into a `movement` and posted to a `lot`.
                            let index_position: usize = 0;
                            let lot_index = vec_of_ordered_index_values[index_position];

                            // Now that the tentative `lot` can be chosen, it is, and a tentative `movement` is created.
                            let lot_to_use = list_of_lots_to_use.borrow()[lot_index].clone();
                            let whole_mvmt = Movement {
                                amount: ar.amount,
                                date_as_string: txn.date_as_string.clone(),
                                date: txn.date,
                                transaction_key: txn_num,
                                action_record_key: *ar_num,
                                cost_basis: Cell::new(d128!(0.0)),
                                ratio_of_amt_to_incoming_mvmts_in_a_r: d128!(1.0),
                                ratio_of_amt_to_outgoing_mvmts_in_a_r: Cell::new(d128!(1.0)),
                                lot_num: lot_to_use.lot_number,
                                proceeds: Cell::new(d128!(0.0)),
                                proceeds_lk: Cell::new(d128!(0.0)),
                                cost_basis_lk: Cell::new(d128!(0.0)),
                            };

                            // Just a last minute check that a home currency `action record` isn't being handled here
                            assert_eq!(raw_acct.is_home_currency(&chosen_home_currency), false);

                            // Beginning here, it will recursively attempt to fit the outgoing amount into `lot`s.
                            fit_into_lots(
                                whole_mvmt,
                                ar.amount,
                                list_of_lots_to_use,
                                vec_of_ordered_index_values,
                                index_position,
                                &chosen_home_currency,
                                &ar,
                                &raw_acct,
                                &acct,
                            );

                            // Once the `action record`'s outgoing amount has been "consumed", the recording of this
                            // `action record` is complete.
                            continue
                        }
                    }

                    // Incoming `action records` have different requirements for posting to `lot`s.  Unlike for outgoing
                    // `action records`, there is often no need to consider how to fit these into lots because in most cases
                    // the amount of an incoming `action record` will be in a single movement that posts to a new `lot`.
                    // There are three exceptions to this which add many lines of code that aren't terribly easy to read. :)
                    // Exception #1: `ToSelf` transactions.  Cost basis and basis date must be preserved for currency
                    // owned by the user and transferred to another one of their accounts.
                    // Exception #2: Like-kind `exchange` `transaction`s must preserve the basis and the basis date
                    // of the corresponding outgoing `action record`.
                    // Exception #3: Dual-`action record` `flow` `transaction`s that occur during a period of like-kind
                    // `exchange` will also inherit an implied/imputed basis date based on the date of the 'buys' in the
                    // base margin `account`.  The special treatment occurs for an incoming `flow` `action record` whose
                    // `account` is non-margin.
                    Polarity::Incoming => {
                        // println!("Txn: {}, Incoming {:?}-type of {} {}",
                        //     txn.tx_number, txn.transaction_type(), ar.amount, acct.ticker);
                        match tx_type {
                            TxType::Flow => {
                                let lot: Rc<Lot>;
                                let mvmt: Movement;
                                // For an incoming `flow` `action record` with a margin account, the implication is that
                                // this is a margin loss `transaction`.  The corresponding outgoing `flow` `action record`
                                // is where the loss is reflected.  This `action record` is simply reflecting the transfer
                                // of funds into the quote margin account, presumably paying off the loan and bringing it
                                // to a zero balance.
                                if raw_acct.is_margin {
                                    let this_acct = acct_map.get(&ar.account_key).unwrap();
                                    let lot_list = this_acct.list_of_lots.borrow_mut();
                                    lot = lot_list.last().unwrap().clone();

                                    mvmt = Movement {
                                        amount: ar.amount,
                                        date_as_string: txn.date_as_string.clone(),
                                        date: txn.date,
                                        transaction_key: txn_num,
                                        action_record_key: *ar_num,
                                        cost_basis: Cell::new(d128!(0.0)),
                                        ratio_of_amt_to_incoming_mvmts_in_a_r: d128!(1.0),
                                        ratio_of_amt_to_outgoing_mvmts_in_a_r: Cell::new(d128!(1.0)),
                                        lot_num: lot.lot_number,
                                        proceeds: Cell::new(d128!(0.0)),
                                        proceeds_lk: Cell::new(d128!(0.0)),
                                        cost_basis_lk: Cell::new(d128!(0.0)),
                                    };
                                    wrap_mvmt_and_push(
                                        mvmt,
                                        &ar,
                                        &lot,
                                        &chosen_home_currency,
                                        &raw_acct,
                                    );

                                    // Since a margin account is being posted new, a new lot is not created.  Once the `movement`
                                    // has been pushed to the `lot`, the recording of the `action record` is complete, and it's
                                    // onto the next
                                    continue

                                // Now the incoming `flow` `action record`s with a non-margin account are handled.
                                } else {

                                    // The base case is a single-`action record` `flow` `transaction` where a `lot` is created (assigned),
                                    // a `movement` is created (assigned), and the `movement` is pushed to the `lot`.  Note that the `lot` variable
                                    // was allocated above, and this `if` section of code merely assigns this `lot` to that variable.
                                    // The `lot` isn't pushed to the `account` until after this whole `if/else` section.
                                    if txn.action_record_idx_vec.len() == 1 {
                                        lot = Rc::new(
                                            Lot {
                                                date_as_string: txn.date_as_string.clone(),
                                                date_of_first_mvmt_in_lot: txn.date,
                                                date_for_basis_purposes: txn.date,

                                                lot_number: length_of_list_of_lots as u32 + 1,
                                                account_key: acct.raw_key,
                                                movements: RefCell::new([].to_vec()),
                                            }
                                        );
                                        mvmt = Movement {
                                            amount: ar.amount,
                                            date_as_string: txn.date_as_string.clone(),
                                            date: txn.date,
                                            transaction_key: txn_num,
                                            action_record_key: *ar_num,
                                            cost_basis: Cell::new(d128!(0.0)),
                                            ratio_of_amt_to_incoming_mvmts_in_a_r: d128!(1.0),
                                            ratio_of_amt_to_outgoing_mvmts_in_a_r: Cell::new(d128!(1.0)),
                                            lot_num: lot.lot_number,
                                            proceeds: Cell::new(d128!(0.0)),
                                            proceeds_lk: Cell::new(d128!(0.0)),
                                            cost_basis_lk: Cell::new(d128!(0.0)),
                                        };

                                    // The more complicated case is the dual-`action record` `flow` `transaction`.
                                    } else {

                                        // A `flow` `transaction` usually has 1 `action record`.  In this special case, it'll have 2, but no more.
                                        assert_eq!(txn.action_record_idx_vec.len(), 2);

                                        // The theory in this `if` block is that a series of margin trades culminating in a margin profit during
                                        // a period of like-kind exchange treatment should/could carry their basis and basis date, just like a traditional
                                        // trade would.  The software allocates the size of the new `movement`s proportionally based on the size of every
                                        // margin buy in the `lot` in relation to all the margin buys in the the `lot`; and for each `movement` that it
                                        // creates, that new `movement` is given the basis date of the respective margin-buy's `movement`.
                                        // (For those savvy, you noted that since margin trades produce no gain/loss, there is no basis to inherit.)
                                        if multiple_incoming_mvmts_per_ar_due_to_lk && txn.date <= like_kind_cutoff_date {
                                            
                                            // First, two variables are allocated to hold some intermediate results that will be used to determine the
                                            // size of `movement`(s) and how many `lot`s are needed.
                                            // The `positive_mvmt_list` is for accumulating the margin-buy `movement`(s) that occurred during the course
                                            // of the margin trade that is now ending in a profit.  And `total_positive_amounts` accounts for the total
                                            // amount of those margin-buys.
                                            let mut positive_mvmt_list: Vec<Rc<Movement>> = [].to_vec();
                                            let mut total_positive_amounts = d128!(0);

                                            // This is necessary to find the base account, because the margin-buys are reflected in this account.
                                            let (base_acct_key, quote_acct_key) = get_base_and_quote_acct_for_dual_actionrecord_flow_tx(
                                                txn_num,
                                                &ar_map,
                                                &raw_acct_map,
                                                &acct_map,
                                                &txns_map,
                                            )?;

                                            let base_acct = acct_map.get(&base_acct_key).unwrap();
                                            let base_acct_lot = base_acct.list_of_lots.borrow().last().unwrap().clone();
                                            // It should be apparent that the relevant `lot` has been selected, and its `movement` are now iterated
                                            // over for capturing its `movement`s (for their date) and adding up their amounts.
                                            for base_acct_mvmt in base_acct_lot.movements.borrow().iter() {
                                                if base_acct_mvmt.amount > d128!(0) {
                                                    // println!("In lot# {}, positive mvmt amount: {} {},",
                                                    //     base_acct_lot.lot_number,
                                                    //     mvmt.borrow().amount,
                                                    //     base_acct_lot.account.raw.ticker);
                                                    total_positive_amounts += base_acct_mvmt.amount;
                                                    positive_mvmt_list.push(base_acct_mvmt.clone())
                                                }
                                            }

                                            // These variables track relevant usage in the following for-loop.  These are used after the for-loop
                                            // when creating the final `movement`.
                                            let mut amounts_used = d128!(0);
                                            let mut percentages_used = d128!(0);

                                            // Here, the margin-buys are iterated over while creating proportionally-sized new `movement`s.
                                            // Note that this for-loop excludes the final positive `movement` because rounding must be taken into
                                            // account (the effect of rounding must be eliminated) when determining the amount of the final `movement`.
                                            // The `inner_lot` and `inner_mvmt` were named this was to reflect they are created and wrapped/pushed
                                            // only inside this iteration of `positive_mvmt_list`.
                                            for pos_mvmt in positive_mvmt_list.iter().take(positive_mvmt_list.len()-1) {
                                                let inner_lot = Rc::new(
                                                    Lot {
                                                        date_as_string: txn.date_as_string.clone(),
                                                        date_of_first_mvmt_in_lot: txn.date,
                                                        date_for_basis_purposes: pos_mvmt.date,
                                                        lot_number: acct.list_of_lots.borrow().len() as u32 + 1,
                                                        account_key: acct.raw_key,
                                                        movements: RefCell::new([].to_vec()),
                                                    }
                                                );
                                                let percentage_used = round_d128_1e8(&(pos_mvmt.amount/&total_positive_amounts));
                                                let amount_used = round_d128_1e8(&(ar.amount*percentage_used));
                                                let inner_mvmt = Movement {
                                                    amount: amount_used,
                                                    date_as_string: txn.date_as_string.clone(),
                                                    date: txn.date,
                                                    transaction_key: txn_num,
                                                    action_record_key: *ar_num,
                                                    cost_basis: Cell::new(d128!(0.0)),
                                                    ratio_of_amt_to_incoming_mvmts_in_a_r: percentage_used,
                                                    ratio_of_amt_to_outgoing_mvmts_in_a_r: Cell::new(d128!(1.0)),
                                                    lot_num: inner_lot.lot_number,
                                                    proceeds: Cell::new(d128!(0.0)),
                                                    proceeds_lk: Cell::new(d128!(0.0)),
                                                    cost_basis_lk: Cell::new(d128!(0.0)),
                                                };
                                                wrap_mvmt_and_push(
                                                    inner_mvmt,
                                                    &ar,
                                                    &inner_lot,
                                                    &chosen_home_currency,
                                                    &raw_acct,
                                                );
                                                acct.list_of_lots.borrow_mut().push(inner_lot);
                                                amounts_used += amount_used;
                                                percentages_used += percentage_used;
                                            }

                                            // Now that the intermediate `lot`s and `movement`s have been taken care of, the `lot` and `movement` that were
                                            // allocated after matching on `flow` can be assigned the following values, and which will be wrapped and
                                            // pushed further down.
                                            let final_pos_mvmt = positive_mvmt_list.last().expect("After exluding last mvmt from for-loop above, expected last mvmt.");
                                            lot = Rc::new(
                                                Lot {
                                                    date_as_string: txn.date_as_string.clone(),
                                                    date_of_first_mvmt_in_lot: txn.date,
                                                    date_for_basis_purposes: final_pos_mvmt.date,
                                                    lot_number: acct.list_of_lots.borrow().len() as u32 + 1,
                                                    account_key: acct.raw_key,
                                                    movements: RefCell::new([].to_vec()),
                                                }
                                            );
                                            mvmt = Movement {
                                                amount: round_d128_1e8(&(ar.amount - amounts_used)),
                                                date_as_string: txn.date_as_string.clone(),
                                                date: txn.date,
                                                transaction_key: txn_num,
                                                action_record_key: *ar_num,
                                                cost_basis: Cell::new(d128!(0.0)),
                                                ratio_of_amt_to_incoming_mvmts_in_a_r: round_d128_1e8(&(d128!(1.0) - percentages_used)),
                                                ratio_of_amt_to_outgoing_mvmts_in_a_r: Cell::new(d128!(1.0)),
                                                lot_num: lot.lot_number,
                                                proceeds: Cell::new(d128!(0.0)),
                                                proceeds_lk: Cell::new(d128!(0.0)),
                                                cost_basis_lk: Cell::new(d128!(0.0)),
                                            };

                                        // Back to "base case" style treatment, if this is an incoming dual-`action record` `flow` `transaction`, but either
                                        // (a) like-kind `exchange` treatment was not elected or (b) the `transaction` date is after the like-kind treatment period,
                                        // then just a single `movement` is created for eventual pushing into a single new `lot`.
                                        } else {
                                            lot = Rc::new(
                                                Lot {
                                                    date_as_string: txn.date_as_string.clone(),
                                                    date_of_first_mvmt_in_lot: txn.date,
                                                    date_for_basis_purposes: txn.date,
                                                    lot_number: length_of_list_of_lots as u32 + 1,
                                                    account_key: acct.raw_key,
                                                    movements: RefCell::new([].to_vec()),
                                                }
                                            );
                                            mvmt = Movement {
                                                amount: ar.amount,
                                                date_as_string: txn.date_as_string.clone(),
                                                date: txn.date,
                                                transaction_key: txn_num,
                                                action_record_key: *ar_num,
                                                cost_basis: Cell::new(d128!(0.0)),
                                                ratio_of_amt_to_incoming_mvmts_in_a_r: d128!(1.0),
                                                ratio_of_amt_to_outgoing_mvmts_in_a_r: Cell::new(d128!(1.0)),
                                                lot_num: lot.lot_number,
                                                proceeds: Cell::new(d128!(0.0)),
                                                proceeds_lk: Cell::new(d128!(0.0)),
                                                cost_basis_lk: Cell::new(d128!(0.0)),
                                            };
                                        }
                                    }

                                    // Here, finally, the lot and movement allocated at the top of `match TxType::Flow` have been set
                                    // and can be wrapped/pushed, at which point this `action record` is complete and it's onto the next.
                                    wrap_mvmt_and_push(
                                        mvmt,
                                        &ar,
                                        &lot,
                                        &chosen_home_currency,
                                        &raw_acct,
                                    );
                                    acct.list_of_lots.borrow_mut().push(lot);
                                    continue
                                }
                            }
                            TxType::Exchange => {

                                // These will only initialize if the outer `if` or inner `if` resolve to false
                                let whole_mvmt;
                                let lot;

                                // The first check is for like-kind exchange treatment is applicable to the `transaction`:
                                if multiple_incoming_mvmts_per_ar_due_to_lk && (txn.date <= like_kind_cutoff_date) {

                                    // If lk is applicable, determine whether to `process_multiple..`,
                                    // based on if each `action record` has a home currency `account`.
                                    let both_are_non_home_curr: bool;
                                    let og_ar = ar_map.get(txn.action_record_idx_vec.first().unwrap()).unwrap();
                                    let og_acct = acct_map.get(&og_ar.account_key).unwrap();
                                    let og_raw_acct = raw_acct_map.get(&og_acct.raw_key).unwrap();
                                    let ic_ar = ar;
                                    let ic_raw_acct = raw_acct;
                                    both_are_non_home_curr = !og_raw_acct.is_home_currency(&chosen_home_currency)
                                    && !ic_raw_acct.is_home_currency(&chosen_home_currency);

                                    if both_are_non_home_curr {
                                        process_multiple_incoming_lots_and_mvmts(
                                            txn_num,
                                            &og_ar,
                                            &ic_ar,
                                            &chosen_home_currency,
                                            &acct_map,
                                            &txns_map,
                                            &ar_map,
                                            &raw_acct,
                                        );
                                        continue

                                    // If lk treatment is applicable but one `account` is home currency, then use a single `lot` and `movement`
                                    } else {
                                        lot = Rc::new(
                                            Lot {
                                                date_as_string: txn.date_as_string.clone(),
                                                date_of_first_mvmt_in_lot: txn.date,
                                                date_for_basis_purposes: txn.date,
                                                lot_number: length_of_list_of_lots as u32 + 1,
                                                account_key: acct.raw_key,
                                                movements: RefCell::new([].to_vec()),
                                            }
                                        );
                                        whole_mvmt = Movement {
                                            amount: ar.amount,
                                            date_as_string: txn.date_as_string.clone(),
                                            date: txn.date,
                                            transaction_key: txn_num,
                                            action_record_key: *ar_num,
                                            cost_basis: Cell::new(d128!(0.0)),
                                            ratio_of_amt_to_incoming_mvmts_in_a_r: d128!(1.0),
                                            ratio_of_amt_to_outgoing_mvmts_in_a_r: Cell::new(d128!(1.0)),
                                            lot_num: lot.lot_number,
                                            proceeds: Cell::new(d128!(0.0)),
                                            proceeds_lk: Cell::new(d128!(0.0)),
                                            cost_basis_lk: Cell::new(d128!(0.0)),
                                        };
                                    }
                                }

                                // For an incoming `action record` in an `exchange` `transaction` where there's no like-kind
                                // treatment, simply create a new `lot`, create a new `movement`, and wrap/push.
                                else {
                                    lot = Rc::new(
                                        Lot {
                                            date_as_string: txn.date_as_string.clone(),
                                            date_of_first_mvmt_in_lot: txn.date,
                                            date_for_basis_purposes: txn.date,
                                            lot_number: length_of_list_of_lots as u32 + 1,
                                            account_key: acct.raw_key,
                                            movements: RefCell::new([].to_vec()),
                                        }
                                    );
                                    whole_mvmt = Movement {
                                        amount: ar.amount,
                                        date_as_string: txn.date_as_string.clone(),
                                        date: txn.date,
                                        transaction_key: txn_num,
                                        action_record_key: *ar_num,
                                        cost_basis: Cell::new(d128!(0.0)),
                                        ratio_of_amt_to_incoming_mvmts_in_a_r: d128!(1.0),
                                        ratio_of_amt_to_outgoing_mvmts_in_a_r: Cell::new(d128!(1.0)),
                                        lot_num: lot.lot_number,
                                        proceeds: Cell::new(d128!(0.0)),
                                        proceeds_lk: Cell::new(d128!(0.0)),
                                        cost_basis_lk: Cell::new(d128!(0.0)),
                                    };
                                }
                                // The `lot` and `whole_mvmt` variables have been initialized/assigned
                                wrap_mvmt_and_push(
                                    whole_mvmt,
                                    &ar,
                                    &lot,
                                    &chosen_home_currency,
                                    &raw_acct,
                                );
                                acct.list_of_lots.borrow_mut().push(lot);
                                continue
                            }
                            TxType::ToSelf => {

                                // Based on experience, and considering how `transaction`s are constructed, this should never happen.
                                if raw_acct.is_margin {
                                    println!("FATAL: Consult developer. Found margin actionrecord in ToSelf transaction:\n{:#?}", txn);
                                    std::process::exit(1);

                                // When transferring to oneself, the amounts should carry over proportionally (considering the incoming `movement`
                                // is likely to be less than the outgoing `movement` due to transaction fees), as should the basis date of each of the
                                // outgoing `movement`s.
                                } else {
                                    process_multiple_incoming_lots_and_mvmts(
                                        txn_num,
                                        &ar_map.get(txn.action_record_idx_vec.first().unwrap()).unwrap(), // outgoing
                                        &ar, // incoming
                                        &chosen_home_currency,
                                        &acct_map,
                                        &txns_map,
                                        &ar_map,
                                        &raw_acct,
                                    );
                                }
                                continue
                            }
                        }   // end for match::TxType
                    }   // end for Polarity::Incoming
                }   // end for match::Polarity
            }   //  end for ar in txn.actionrecords (ar_num in tx.ar_idx_vec)
        }   //  end of tx does not have marginness of TwoARs
    }   //  end for txn in transactions (txn_num in txn_map.len())
    Ok(txns_map)
}

/// Preface: this ONLY works for a dual-`action record` `transaction` when the `account` of the incoming
/// `action record` is a non-margin `account`.  Also, we know that the corresponding outgoing `action
/// record` is the quote account (logically, it must be).
///
/// This function works off of an incoming `action record` for a non-margin account. It knows the outgoing 
/// `action record` of this `transaction` has a margin account, so it first will
/// choose that `action record`, and then it'll immediately choose the margin `account` in question.
/// Since the margin pair never changes (the same base `account` and same quote `account` interact exclusively
/// with eachother), it can immediately get the first `lot` in the `account` (to ensure it exists).
/// Then it gets a list of `movement`s in that first `lot`.  Then, to ensure it's getting a `transaction`
/// where both `action record`s have a margin account, it chooses the first `movement`.  And then it
/// takes that `transaction` to analyze for base `account` and quote `account`.
fn get_base_and_quote_acct_for_dual_actionrecord_flow_tx(
    txn_num: u32,
    ar_map: &HashMap<u32, ActionRecord>,
    raw_acct_map: &HashMap<u16, RawAccount>,
    acct_map: &HashMap<u16, Account>,
    txns_map: &HashMap<u32, Transaction>,
) -> Result<(u16, u16), Box<dyn Error>> {

    let txn = txns_map.get(&txn_num).expect("Couldn't get txn. Tx num invalid?");

    let og_flow_ar = ar_map.get(txn.action_record_idx_vec.first().unwrap()).unwrap();
    let og_flow_ar_acct = acct_map.get(&og_flow_ar.account_key).unwrap();
    let og_flow_ar_acct_first_lot = &og_flow_ar_acct.list_of_lots.borrow()[0];
    let first_lot_mvmts = og_flow_ar_acct_first_lot.movements.borrow();
    let first_lot_mvmts_first_mvmt = &first_lot_mvmts.first().unwrap();
    let txn_of_first_lot_mvmts_first_mvmt = txns_map.get(&first_lot_mvmts_first_mvmt.transaction_key).unwrap();

    let (base_key,quote_key) = txn_of_first_lot_mvmts_first_mvmt.get_base_and_quote_raw_acct_keys(
        ar_map,
        &raw_acct_map,
        &acct_map)?;
	Ok((base_key, quote_key))
}

/// Returns the index in the `action records` Hashmap of the base and quote `action record`s.
fn get_base_and_quote_ar_idxs(
    base_and_quote_keys: (u16,u16),
    txn: &Transaction,
    ars: &HashMap<u32, ActionRecord>,
    raw_acct_map: &HashMap<u16, RawAccount>,
    acct_map: &HashMap<u16, Account>
    ) -> (u32, u32) {

    let incoming_ar = ars.get(&txn.action_record_idx_vec[0]).unwrap();
    let incoming_acct = acct_map.get(&incoming_ar.account_key).unwrap();
    let raw_ic_acct = raw_acct_map.get(&incoming_acct.raw_key).unwrap();
    let compare = raw_acct_map.get(&base_and_quote_keys.0).unwrap();  //  key.0 is base, and key.1 is quote

    if raw_ic_acct == compare {
        (txn.action_record_idx_vec[0], txn.action_record_idx_vec[1])
    } else {
        (txn.action_record_idx_vec[1], txn.action_record_idx_vec[0])
    }
}

/// Every time a new `movement` is created, it must be wrapped in an `Rc` because it is owned both by the
/// `lot` (which itself is owned by an `account`) and by the `action record` from which it was derived.
fn wrap_mvmt_and_push(
    this_mvmt: Movement,
    ar: &ActionRecord,
    lot: &Lot,
    chosen_home_currency: &str,
    raw_acct: &RawAccount,
) {

    // For outgoing `action record`s, this is an optimal spot for setting this struct field. Interestingly,
    // at the time of writing this note, this ratio isn't actually used.  The `ratio_of_amt_to_incoming_mvmts_in_a_r`
    // field, by contrast, is extremely important when deterministically setting basis and proceeds.
    // TODO: Consider commenting or deleting this code block (and the two vars above).
    if ar.direction() == Polarity::Outgoing && !raw_acct.is_home_currency(chosen_home_currency) {
        let ratio = this_mvmt.amount / ar.amount;
        this_mvmt.ratio_of_amt_to_outgoing_mvmts_in_a_r.set(round_d128_1e8(&ratio));
    }

    // This is the type of thing that should probably be changed into a test or an operation that is only
    // run under certain circumstances, since it is double-checking something that prior code should have
    // already taken care of, which is that the value has been rounded to 8 digits of precision already.
    // TODO: Consider deleting this assertion.
    let amt = this_mvmt.amount;
    let amt2 = round_d128_1e8(&amt);
    assert_eq!(amt, amt2);

    let mvmt = Rc::from(this_mvmt);
    lot.movements.borrow_mut().push(mvmt.clone());
    ar.movements.borrow_mut().push(mvmt);
}

/// Recursively check the balance in a `lot`, and if not zero then create a `movement` that is the lesser of
/// the `mvmt_to_fit` or the balance of the `lot`; and if the `lot` balance is smaller than the amount of
/// the `mvmt_to_fit`, then create a `movement` that will fit into that `lot` and push it to that `lot`; and
/// then select the next `lot` and replace the `mvmt_to_fit` with a new `mvmt_to_fit` (reduced by the one
/// that was pushed to the previous `lot`), and recursively check...
fn fit_into_lots(
    mvmt_to_fit: Movement,
    amt_to_fit: d128,
    list_of_lots_to_use: RefCell<Vec<Rc<Lot>>>,
    vec_of_ordered_index_values: Vec<usize>,
    index_position: usize,
    chosen_home_currency: &str,
    ar: &ActionRecord,
    raw_acct: &RawAccount,
    acct: &Account,
) {

    let mut current_index_position = index_position;

    // Here is a check to make sure the `lot` will exist. If it won't, then there will be an index
    // out of bounds error. The account balance should be zero in that case, but it is checked
    // anyway before printing the error message for the user and exiting.
    if vec_of_ordered_index_values.len() == current_index_position {
        println!("FATAL: Txn {} on {} spending {} {} has run out of lots to spend from.",
            mvmt_to_fit.transaction_key, mvmt_to_fit.date_as_string, ar.amount, raw_acct.ticker);
        let bal = if acct.get_sum_of_amts_in_lots() == d128!(0) { "0.00000000".to_string() } 
            else { acct.get_sum_of_amts_in_lots().to_string() };
        println!("Account balance is only: {}", bal);
        std::process::exit(1);
    }

    // Get the `lot`, and then get its balance to see how much room there is
    let lot_index = vec_of_ordered_index_values[current_index_position];
    let lot = acct.list_of_lots.borrow()[lot_index].clone();
    let mut sum_of_mvmts_in_lot: d128 = d128!(0.0);
    for movement in lot.movements.borrow().iter() {
        sum_of_mvmts_in_lot += movement.amount;
    }

    assert!(sum_of_mvmts_in_lot >= d128!(0.0));

    //  If the `lot` is "full", try the next.
    if sum_of_mvmts_in_lot == d128!(0.0) {

        current_index_position += 1;

        fit_into_lots(
            mvmt_to_fit,
            amt_to_fit,
            list_of_lots_to_use,
            vec_of_ordered_index_values,
            current_index_position,
            chosen_home_currency,
            ar,
            raw_acct,
            acct,
        );
        return;
    }

    assert!(sum_of_mvmts_in_lot > d128!(0.0));

    // If `remainder_amt_to_recurse` is positive, it means the `lot` balance exceeded `amt_to_fit`,
    // therefore, the amount completely fits in the `lot`.  If negative, it is passed as the `amt_to_fit`
    // for the next round of recursion.
    let remainder_amt_to_recurse = (amt_to_fit + sum_of_mvmts_in_lot).reduce();

    // If the remainder fits, the `movement` is wrapped/pushed, and the recursion is complete.
    if remainder_amt_to_recurse >= d128!(0.0) {

        let remainder_mvmt_that_fits: Movement = Movement {
            amount: amt_to_fit,
            lot_num: lot.lot_number,
            ..mvmt_to_fit
        };
        wrap_mvmt_and_push(
            remainder_mvmt_that_fits,
            &ar,
            &lot,
            &chosen_home_currency,
            &raw_acct
        );
        return
    }

    // The amt_to_fit doesn't completely fit in the present `lot`, but some does. Create a `movement` that will fit.
    let mvmt_that_fits_in_lot: Movement = Movement {
        amount: (-sum_of_mvmts_in_lot).reduce(),
        lot_num: lot.lot_number,
        ..mvmt_to_fit.clone()
    };
    wrap_mvmt_and_push(
        mvmt_that_fits_in_lot,
        &ar,
        &lot,
        &chosen_home_currency,
        &raw_acct
    );

    current_index_position += 1;

    // After applying some of the `amt_to_fit` to the `lot`, increment the index, take the remainder, and recurse
    fit_into_lots(
        mvmt_to_fit,
        remainder_amt_to_recurse.reduce(),  //  This was updated before recursing
        list_of_lots_to_use,
        vec_of_ordered_index_values,
        current_index_position,     //  This was updated before recursing
        chosen_home_currency,
        ar,
        raw_acct,
        acct,
    );
}

/// This is for the surprisingly common occasion (not surprising once you think about it) when an
/// incoming `action record` must be split into multiple `movement`s and therefore multiple `lot`s.
/// This happens every time a user transfers from one account of theirs to another.
fn process_multiple_incoming_lots_and_mvmts(
    txn_num: u32,
    outgoing_ar: &ActionRecord,
    incoming_ar: &ActionRecord,
    chosen_home_currency: &str,
    acct_map: &HashMap<u16, Account>,
    txns_map: &HashMap<u32, Transaction>,
    ar_map: &HashMap<u32, ActionRecord>,
    raw_acct: &RawAccount,
) {

    let round_to_places = d128::from(1).scaleb(d128::from(-8));
    let txn = txns_map.get(&txn_num).expect("Couldn't get txn. Tx num invalid?");

    let acct_of_incoming_ar = acct_map.get(&incoming_ar.account_key).unwrap();

    let mut all_but_last_incoming_mvmt_amt = d128!(0.0);
    let mut all_but_last_incoming_mvmt_ratio = d128!(0.0);
    // println!("Txn date: {}. Outgoing mvmts: {}, Outgoing amount: {}", txn.date, outgoing_ar.movements.borrow().len(), outgoing_ar.amount);
    let list_of_mvmts_of_outgoing_ar = outgoing_ar.get_mvmts_in_ar_in_lot_date_order(acct_map, txns_map);
    let list_of_mvmts_of_outgoing_ar_len = list_of_mvmts_of_outgoing_ar.len();
    let final_og_mvmt = list_of_mvmts_of_outgoing_ar.last().unwrap();
    //  First iteration, for all but final movement
    for outgoing_mvmt in list_of_mvmts_of_outgoing_ar.iter().take(list_of_mvmts_of_outgoing_ar_len - 1) {
        let ratio_of_outgoing_mvmt_to_total_ar = outgoing_mvmt.amount / outgoing_ar.amount; //  Negative divided by negative is positive
        // println!("Ratio of outgoing amt to total actionrecord amt: {:.8}", ratio_of_outgoing_to_total_ar);
        let tentative_incoming_amt = ratio_of_outgoing_mvmt_to_total_ar * incoming_ar.amount;
        // println!("Unrounded incoming amt: {}", tentative_incoming_amt);
        let corresponding_incoming_amt = tentative_incoming_amt.quantize(round_to_places);
        // println!("Rounded incoming amt: {}", corresponding_incoming_amt);
        if corresponding_incoming_amt == d128!(0) { continue }  //  Due to rounding, this could be zero.
        assert!(corresponding_incoming_amt > d128!(0.0));
        let this_acct = acct_of_incoming_ar;
        let length_of_list_of_lots: usize = this_acct.list_of_lots.borrow().len();
        let inherited_date = outgoing_mvmt.get_lot(acct_map, ar_map).date_of_first_mvmt_in_lot;
        let inner_lot =
        Rc::new(
            Lot {
                date_as_string: txn.date_as_string.clone(),
                date_of_first_mvmt_in_lot: txn.date,
                date_for_basis_purposes: inherited_date,
                lot_number: length_of_list_of_lots as u32 + 1,
                account_key: this_acct.raw_key,
                movements: RefCell::new([].to_vec()),
            }
        )
        ;
        let incoming_mvmt = Movement {
            amount: corresponding_incoming_amt.reduce(),
            date_as_string: txn.date_as_string.clone(),
            date: txn.date,
            transaction_key: txn_num,
            action_record_key: incoming_ar.self_ar_key,
            cost_basis: Cell::new(d128!(0.0)),
            ratio_of_amt_to_incoming_mvmts_in_a_r: round_d128_1e8(&ratio_of_outgoing_mvmt_to_total_ar),
            ratio_of_amt_to_outgoing_mvmts_in_a_r: Cell::new(d128!(1.0)),
            lot_num: inner_lot.lot_number,
            proceeds: Cell::new(d128!(0.0)),
            proceeds_lk: Cell::new(d128!(0.0)),
            cost_basis_lk: Cell::new(d128!(0.0)),
        };
        // println!("From first set of incoming movements, amount: {} {} to account: {}",
        //     incoming_mvmt.amount, acct_incoming_ar.ticker, acct_incoming_ar.account_num);
        all_but_last_incoming_mvmt_ratio += round_d128_1e8(&ratio_of_outgoing_mvmt_to_total_ar);
        all_but_last_incoming_mvmt_amt += incoming_mvmt.amount;
        wrap_mvmt_and_push(
            incoming_mvmt,
            &incoming_ar,
            &inner_lot,
            &chosen_home_currency,
            &raw_acct,
        );
        this_acct.list_of_lots.borrow_mut().push(inner_lot);
    }
    //  Second iteration, for final movement
    let corresponding_incoming_amt = incoming_ar.amount - all_but_last_incoming_mvmt_amt;
    assert!(corresponding_incoming_amt > d128!(0.0));
    let this_acct = acct_of_incoming_ar;
    let length_of_list_of_lots = this_acct.list_of_lots.borrow().len();
    let inherited_date = final_og_mvmt.get_lot(acct_map, ar_map).date_of_first_mvmt_in_lot;
    let lot =
    Rc::new(
        Lot {
            date_as_string: txn.date_as_string.clone(),
            date_of_first_mvmt_in_lot: txn.date,
            date_for_basis_purposes: inherited_date,
            lot_number: length_of_list_of_lots as u32 + 1,
            account_key: this_acct.raw_key,
            movements: RefCell::new([].to_vec()),
        }
    )
    ;
    let incoming_mvmt = Movement {
        amount: corresponding_incoming_amt.reduce(),
        date_as_string: txn.date_as_string.clone(),
        date: txn.date,
        transaction_key: txn_num,
        action_record_key: incoming_ar.self_ar_key,
        cost_basis: Cell::new(d128!(0.0)),
        ratio_of_amt_to_incoming_mvmts_in_a_r: d128!(1.0) - all_but_last_incoming_mvmt_ratio,
        ratio_of_amt_to_outgoing_mvmts_in_a_r: Cell::new(d128!(1.0)),
        lot_num: lot.lot_number,
        proceeds: Cell::new(d128!(0.0)),
        proceeds_lk: Cell::new(d128!(0.0)),
        cost_basis_lk: Cell::new(d128!(0.0)),
    };
    // println!("Final incoming mvmt for this actionrecord, amount: {} {} to account: {}",
    //     incoming_mvmt.amount, acct_incoming_ar.ticker, acct_incoming_ar.account_num);
    wrap_mvmt_and_push(
        incoming_mvmt,
        &incoming_ar,
        &lot,
        &chosen_home_currency,
        &raw_acct,
    );
    this_acct.list_of_lots.borrow_mut().push(lot);
}
