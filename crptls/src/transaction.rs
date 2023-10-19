// Copyright (c) 2017-2020, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools/blob/master/LEGAL.txt

use std::rc::Rc;
use std::cell::RefCell;
use std::process;
use std::fmt;
use std::collections::HashMap;
use std::error::Error;

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use chrono::NaiveDate;
use serde_derive::{Serialize, Deserialize};

use crate::account::{Account, Movement, RawAccount};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
	pub tx_number: u32,	//	Does NOT start at zero.  First txn is 1.
	pub date_as_string: String,
	pub date: NaiveDate,
	pub user_memo: String,
	pub proceeds: f32,
	pub action_record_idx_vec: Vec<u32>,
}

impl Transaction {

	pub fn transaction_type(
		&self,
		ars: &HashMap<u32, ActionRecord>,
		raw_acct_map: &HashMap<u16, RawAccount>,
		acct_map: &HashMap<u16, Account>,
	) -> Result<TxType, Box<dyn Error>> {

		if self.action_record_idx_vec.len() == 1 {
			Ok(TxType::Flow)
		}
		else if self.action_record_idx_vec.len() == 2 {
			//	This exercise of splitting the strings is because of margin accounts, where BTC borrowed to buy XMR would reflect as BTC_xmr
			let first_ar = ars.get(&self.action_record_idx_vec[0]).unwrap();
			let second_ar = ars.get(&self.action_record_idx_vec[1]).unwrap();
			let first_acct = acct_map.get(&first_ar.account_key).unwrap();
			let second_acct = acct_map.get(&second_ar.account_key).unwrap();
			let ar1_raw_acct = raw_acct_map.get(&first_acct.raw_key).unwrap();
			let ar2_raw_acct = raw_acct_map.get(&second_acct.raw_key).unwrap();
			let ar1_ticker_full = ar1_raw_acct.ticker.clone();
			let ar2_ticker_full = ar2_raw_acct.ticker.clone();
			let ar1_ticker_comp: Vec<&str> = ar1_ticker_full.split('_').collect();
			let ar2_ticker_comp: Vec<&str> = ar2_ticker_full.split('_').collect();
			let ar1_ticker = ar1_ticker_comp[0];
			let ar2_ticker = ar2_ticker_comp[0];

			if first_ar.direction() == second_ar.direction() {
                println!("FATAL: TxType: Found transaction with two actionRecords with the same polarity: \n{:#?}", self);
                process::exit(1);
			}
			if ar1_ticker == ar2_ticker {
				if ar1_raw_acct.is_margin != ar2_raw_acct.is_margin {
					Ok(TxType::Flow)
				}
				else {
					Ok(TxType::ToSelf)
				}
			}
			else {
				Ok(TxType::Exchange)
			}
		}
		else if self.action_record_idx_vec.len() > 2 {
            println!("FATAL: TxType: Found transaction with too many actionRecords: \n{:#?}", self);
            process::exit(1);
		}
		else {
            println!("FATAL: TxType: Found transaction with no actionRecords: \n{:#?}", self);
            process::exit(1);
		}
	}

	pub fn marginness(
		&self,
		ars: &HashMap<u32, ActionRecord>,
		raw_acct_map: &HashMap<u16, RawAccount>,
		acct_map: &HashMap<u16, Account>
	) -> TxHasMargin {

		if self.action_record_idx_vec.len() == 1 {
			let ar = ars.get(&self.action_record_idx_vec[0]).unwrap();
			let acct = acct_map.get(&ar.account_key).unwrap();
			let raw_acct = raw_acct_map.get(&acct.raw_key).unwrap();
			if raw_acct.is_margin {
				TxHasMargin::OneAR
			} else {
				TxHasMargin::NoARs
			}
		} else {

            if self.action_record_idx_vec.len() != 2 {
                println!("FATAL: Each transaction may have up to two actionrecords, and there are {} actionrecords in transaction:\n{:#?}",
                self.action_record_idx_vec.len(), self);
            }

            let first_ar = match ars.get(&self.action_record_idx_vec[0]) {
                Some(x) => x,
                None => {
                    println!("FATAL: ActionRecord not found for: \n{:#?}", self);
                    process::exit(1)}
            };
			let second_ar = match ars.get(&self.action_record_idx_vec[1]) {
                Some(x) => x,
                None => {
                    println!("FATAL: ActionRecord not found for: \n{:#?}", self);
                    process::exit(1)}
            };

			let first_acct = acct_map.get(&first_ar.account_key).unwrap();
			let second_acct = acct_map.get(&second_ar.account_key).unwrap();

			let first_raw_acct = &raw_acct_map.get(&first_acct.raw_key).unwrap();
			let second_raw_acct = &raw_acct_map.get(&second_acct.raw_key).unwrap();

			if first_raw_acct.is_margin {
				if second_raw_acct.is_margin {TxHasMargin::TwoARs} else {TxHasMargin::OneAR}
			} else if second_raw_acct.is_margin {TxHasMargin::OneAR} else {TxHasMargin::NoARs}
		}
	}

	pub fn get_base_and_quote_raw_acct_keys(
		&self,
		ars: &HashMap<u32, ActionRecord>,
		raw_accts: &HashMap<u16, RawAccount>,
		acct_map: &HashMap<u16, Account>
	) -> Result<(u16, u16), Box<dyn Error>> {

		assert_eq!(self.transaction_type(ars, raw_accts, acct_map)?, TxType::Exchange,
			"This can only be called on exchange transactions.");

		let first_ar = ars.get(&self.action_record_idx_vec[0]).unwrap();
		let second_ar = ars.get(&self.action_record_idx_vec[1]).unwrap();
		let first_acct = acct_map.get(&first_ar.account_key).unwrap();
		let second_acct = acct_map.get(&second_ar.account_key).unwrap();
		let first_acct_raw_key = first_acct.raw_key;
		let second_acct_raw_key = second_acct.raw_key;
		let first_raw_acct = raw_accts.get(&first_acct_raw_key).unwrap();
		let second_raw_acct = raw_accts.get(&second_acct_raw_key).unwrap();

		assert_eq!(first_raw_acct.is_margin, true, "First actionrecord wasn't a margin account. Both must be.");
		assert_eq!(second_raw_acct.is_margin, true, "Second actionrecord wasn't a margin account. Both must be.");

		let quote: u16;
		let base: u16;

        if first_raw_acct.ticker.contains('_') {
			quote = first_acct_raw_key;
			base = second_acct_raw_key;
			Ok((base, quote))
		} else if second_raw_acct.ticker.contains('_') {
			base = first_acct_raw_key;
			quote = second_acct_raw_key;
			Ok((base, quote))
		} else {
            println!("FATAL: {}", VariousErrors::MarginNoUnderbar);
            std::process::exit(1);
		}
	}

	pub fn get_outgoing_exchange_and_flow_mvmts(
        &self,
        user_home_currency: &str,
        ars: &HashMap<u32, ActionRecord>,
        raw_acct_map: &HashMap<u16, RawAccount>,
        acct_map: &HashMap<u16, Account>,
        txns_map: &HashMap<u32, Transaction>,
    ) -> Result<Vec<Rc<Movement>>, Box<dyn Error>> {

		let mut flow_or_outgoing_exchange_movements = [].to_vec();

        for ar_num in self.action_record_idx_vec.iter() {

            let ar = ars.get(ar_num).unwrap();
            let acct = acct_map.get(&ar.account_key).unwrap();
            let raw_acct = raw_acct_map.get(&acct.raw_key).unwrap();

            if !raw_acct.is_home_currency(user_home_currency) & !raw_acct.is_margin {

                let movements = ar.get_mvmts_in_ar_in_lot_date_order(acct_map, txns_map);

                match self.transaction_type(ars, raw_acct_map, acct_map)? {
                    TxType::Exchange => {
                        if Polarity::Outgoing == ar.direction() {
                            for mvmt in movements.iter() {
                                flow_or_outgoing_exchange_movements.push(mvmt.clone());
                            }
                        }
                    }
                    TxType::Flow => {
                        for mvmt in movements.iter() {
                            flow_or_outgoing_exchange_movements.push(mvmt.clone());
                        }
                    }
                    _ => {}
                }
            }
        }
		Ok(flow_or_outgoing_exchange_movements)
	}

    pub fn both_exch_ars_are_non_home_curr(
        &self,
        ars: &HashMap<u32, ActionRecord>,
        raw_acct_map: &HashMap<u16, RawAccount>,
        acct_map: &HashMap<u16, Account>,
        home_currency: &str,
    ) -> Result<bool, Box<dyn Error>> {

        assert_eq!(self.action_record_idx_vec.len(), (2 as usize));

        let og_ar = ars.get(&self.action_record_idx_vec.first().unwrap()).unwrap();
        let ic_ar = ars.get(&self.action_record_idx_vec.last().unwrap()).unwrap();
        let og_acct = acct_map.get(&og_ar.account_key).unwrap();
        let ic_acct = acct_map.get(&ic_ar.account_key).unwrap();
        let raw_og_acct = raw_acct_map.get(&og_acct.raw_key).unwrap();
        let raw_ic_acct = raw_acct_map.get(&ic_acct.raw_key).unwrap();

        let og_is_home_curr = raw_og_acct.is_home_currency(&home_currency);
        let ic_is_home_curr = raw_ic_acct.is_home_currency(&home_currency);
        let both_are_non_home_curr = !ic_is_home_curr && !og_is_home_curr;

        Ok(both_are_non_home_curr)
    }

    pub fn get_auto_memo(
		&self,
		ars: &HashMap<u32, ActionRecord>,
		raw_accts: &HashMap<u16, RawAccount>,
		acct_map: &HashMap<u16, Account>,
        home_currency: &str,
	) -> Result<String, Box<dyn Error>> {

        let auto_memo = if self.action_record_idx_vec.len() == 2 {

            let tx_type = self.transaction_type(ars, raw_accts, acct_map)?;

            let marginness = self.marginness(ars, raw_accts, acct_map);

            if (marginness == TxHasMargin::NoARs) | (marginness == TxHasMargin::TwoARs)  {

                let og_amt = ars.get(&self.action_record_idx_vec[0]).unwrap().amount;
                let og_acct_key = ars.get(&self.action_record_idx_vec[0]).unwrap().account_key;
                let og_acct = acct_map.get(&og_acct_key).unwrap();
                let og_raw_acct = raw_accts.get(&og_acct.raw_key).unwrap();
                let og_ticker = &og_raw_acct.ticker;

                let ic_amt = ars.get(&self.action_record_idx_vec[1]).unwrap().amount;
                let ic_acct_key = ars.get(&self.action_record_idx_vec[1]).unwrap().account_key;
                let ic_acct = acct_map.get(&ic_acct_key).unwrap();
                let ic_raw_acct = raw_accts.get(&ic_acct.raw_key).unwrap();
                let ic_ticker = &ic_raw_acct.ticker;

                let og_amt_and_ticker;
                if og_raw_acct.is_home_currency(home_currency) {
                    og_amt_and_ticker = format!("{:.2} {}",
                        og_amt.to_string().as_str().parse::<f32>()?, og_ticker
                    );
                } else {
                    og_amt_and_ticker = format!("{} {}", og_amt, og_ticker);
                }

                let ic_amt_and_ticker;
                if ic_raw_acct.is_home_currency(home_currency) {
                    ic_amt_and_ticker = format!("{:.2} {}",
                        ic_amt.to_string().as_str().parse::<f32>()?, ic_ticker
                    );
                } else {
                    ic_amt_and_ticker = format!("{} {}", ic_amt, ic_ticker);
                }

                if tx_type == TxType::Exchange {
                    format!("Paid {} for {}, valued at {:.2} {}.",
                        og_amt_and_ticker, ic_amt_and_ticker,
                        self.proceeds.to_string().as_str().parse::<f32>()?, home_currency)
                } else {
                    format!("Transferred {} to another account. Received {}, likely after a transaction fee.",
                        og_amt_and_ticker, ic_amt_and_ticker)
                }
            } else {

                format!("Margin profit or loss valued at {:.2} {}.",
                self.proceeds.to_string().as_str().parse::<f32>()?, home_currency)
            }

        } else {

            let amt = ars.get(&self.action_record_idx_vec[0]).unwrap().amount;
            let acct_key = ars.get(&self.action_record_idx_vec[0]).unwrap().account_key;
            let acct = acct_map.get(&acct_key).unwrap();
            let raw_acct = raw_accts.get(&acct.raw_key).unwrap();
            let ticker = &raw_acct.ticker;

            if amt > dec!(0.0) {

                format!("Received {} {} valued at {:.2} {}.", amt, ticker,
                self.proceeds.to_string().as_str().parse::<f32>()?, home_currency)

            } else {

                format!("Spent {} {} valued at {:.2} {}.", amt, ticker,
                self.proceeds.to_string().as_str().parse::<f32>()?, home_currency)

            }
        };

        Ok(auto_memo)
    }
}

#[derive(Clone, Debug)]
pub struct ActionRecord {
	pub account_key: u16,
	pub amount: Decimal,
    pub tx_key: u32,
    pub self_ar_key: u32,
	pub movements: RefCell<Vec<Rc<Movement>>>,
}

impl ActionRecord {

	pub fn direction(&self) -> Polarity {
		if self.amount < dec!(0.0) { Polarity::Outgoing}
		else { Polarity::Incoming }
    }

    pub fn cost_basis_in_ar(&self) -> Decimal {

        let mut cb = dec!(0);

        for mvmt in self.movements.borrow().iter() {
            cb += mvmt.cost_basis.get()
        }

        cb.abs()
    }

	// pub fn is_quote_acct_for_margin_exch(
	// 	&self,
	// 	raw_accts: &HashMap<u16, RawAccount>,
	// 	acct_map: &HashMap<u16, Account>
	// ) -> bool {

	// 	let acct = acct_map.get(&self.account_key).unwrap();
	// 	let raw_acct = raw_accts.get(&acct.raw_key).unwrap();
	// 	raw_acct.ticker.contains('_')
	// }

    /// Iterates through every `Lot` in the `list_of_lots` of the `ActionRecord`'s `Account`
    /// until it finds all the `Movements` - cloning each along the way - and then returns
    /// a `Vec` of `Rc<Movements>`.
    ///
    /// Note that a `Lot`'s `date`, and generally its `basis_date` too, will increase
    /// chronologically along with the `Lot`'s `lot_num` which is just it's `index` in the
    /// `list_of_lots` plus `1`. Exceptions will occur, because `Lot`s are permanently
    /// ordered by their creation date (`date`), so later `Lot`s may have earlier `basis_date`s
    /// by virtue of them being the result of a `ToSelf` type `Transaction` that transferred
    /// old "coins."
    pub fn get_mvmts_in_ar_in_lot_date_order(
        &self,
        acct_map: &HashMap<u16, Account>,
        txns_map: &HashMap<u32, Transaction>,
    ) -> Vec<Rc<Movement>> {

        let txn = txns_map.get(&self.tx_key).unwrap();
        let mut movements_in_ar = [].to_vec();
        let acct = acct_map.get(&self.account_key).unwrap();

        let target = self.amount;
        let mut measure = dec!(0);

        for lot in acct.list_of_lots.borrow().iter() {

            for mvmt in lot.movements.borrow().iter() {

                if (mvmt.date) <= txn.date && mvmt.action_record_key == self.self_ar_key {

                    measure += mvmt.amount;

                    movements_in_ar.push(mvmt.clone());

                    if measure == target { return movements_in_ar }
                }
            }
        }
        println!("ERROR: This should never print.");
        movements_in_ar
	}
}

#[derive(Clone, Debug, PartialEq)]
pub enum TxType {
	Exchange,
	ToSelf,
	Flow,
}

impl fmt::Display for TxType {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TxType::Exchange => write!(f, "Exchange"),
            TxType::ToSelf => write!(f, "ToSelf"),
            TxType::Flow => write!(f, "Flow"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TxHasMargin {
	NoARs,
	OneAR,
	TwoARs,
}

impl fmt::Display for TxHasMargin {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       match *self {
           TxHasMargin::NoARs => write!(f, "No actionrecord is a margin account"),
           TxHasMargin::OneAR => write!(f, "One actionrecord is a margin account"),
           TxHasMargin::TwoARs => write!(f, "Two actionrecords are margin accounts"),
       }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Polarity {
	Outgoing,
	Incoming,
}

impl fmt::Display for Polarity {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       match *self {
           Polarity::Outgoing => write!(f, "Outgoing"),
           Polarity::Incoming => write!(f, "Incoming"),
       }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum VariousErrors {
	MarginNoUnderbar,
}

impl fmt::Display for VariousErrors {

	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			VariousErrors::MarginNoUnderbar => write!(f,
			"Neither account ticker contained an underbar '_', so the quote account couldn't be determined.
			For example, for the 'USD/EUR' pair, USD is the base account and EUR is the quote account.
			In order for this software to function correctly, the quote ticker should be denoted as 'EUR_usd'.")
		}
	}
}
