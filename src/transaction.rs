// Copyright (c) 2017-2019, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools-rs/LEGAL.txt

use std::rc::{Rc};
use std::cell::{RefCell};
use std::process::exit;
use std::fmt;
use std::collections::{HashMap};

use decimal::d128;
use chrono::NaiveDate;
use serde_derive::{Serialize, Deserialize};

use crate::cli_user_choices::LotProcessingChoices;
use crate::account::{Account, Movement, RawAccount};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
	pub tx_number: u32,	//	Does NOT start at zero.  First txn is 1.
	pub date_as_string: String,
	pub date: NaiveDate,
	pub memo: String,
	pub proceeds: f32,
	pub action_record_idx_vec: Vec<u32>,
}

impl Transaction {

	pub fn transaction_type(
		&self,
		ars: &HashMap<u32, ActionRecord>,
		raw_acct_map: &HashMap<u16, RawAccount>,
		acct_map: &HashMap<u16, Account>,
	) -> TxType {

		if self.action_record_idx_vec.len() == 1 {
			TxType::Flow
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
				println!("Program exiting. Found transaction with two actionRecords with the same polarity: {:?}", self); exit(1);
			}
			if ar1_ticker == ar2_ticker {
				if ar1_raw_acct.is_margin != ar2_raw_acct.is_margin {
					TxType::Flow
				}
				else {
					TxType::ToSelf
				}
			}
			else {
				TxType::Exchange
			}
		}
		else if self.action_record_idx_vec.len() > 2 {
			println!("Program exiting. Found transaction with too many actionRecords: {:?}", self); exit(1);
		}
		else {
			println!("Program exiting. Found transaction with no actionRecords: {:?}", self); exit(1);
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
			assert_eq!(self.action_record_idx_vec.len(),2);
			let first_ar = ars.get(&self.action_record_idx_vec[0]).unwrap();
			let second_ar = ars.get(&self.action_record_idx_vec[1]).unwrap();

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
	) -> (u16, u16) {

		assert_eq!(self.transaction_type(ars, raw_accts, acct_map), TxType::Exchange,
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
			(base, quote)
		} else if second_raw_acct.ticker.contains('_') {
			base = first_acct_raw_key;
			quote = second_acct_raw_key;
			(base, quote)
		} else {
			println!("{}", VariousErrors::MarginNoUnderbar); use std::process::exit; exit(1)
		}
	}

	pub fn get_outgoing_exchange_and_flow_mvmts(
        &self,
        settings: &LotProcessingChoices,
        ars: &HashMap<u32, ActionRecord>,
        raw_acct_map: &HashMap<u16, RawAccount>,
        acct_map: &HashMap<u16, Account>,
        txns_map: &HashMap<u32, Transaction>,
    ) -> Vec<Rc<Movement>> {

		let mut flow_or_outgoing_exchange_movements = [].to_vec();

        for ar_num in self.action_record_idx_vec.iter() {

            let ar = ars.get(ar_num).unwrap();
            let acct = acct_map.get(&ar.account_key).unwrap();
            let raw_acct = raw_acct_map.get(&acct.raw_key).unwrap();

            if !raw_acct.is_home_currency(&settings.home_currency) & !raw_acct.is_margin {

                let movements = ar.get_mvmts_in_ar(acct_map, txns_map);

                match self.transaction_type(ars, raw_acct_map, acct_map) {
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
		flow_or_outgoing_exchange_movements
	}
}

#[derive(Clone, Debug)]
pub struct ActionRecord {
	pub account_key: u16,
	pub amount: d128,
    pub tx_key: u32,
    pub self_ar_key: u32,
	pub movements: RefCell<Vec<Rc<Movement>>>,
}

impl ActionRecord {

	pub fn direction(&self) -> Polarity {
		if self.amount < d128!(0.0) { Polarity::Outgoing}
		else { Polarity::Incoming }
	}

	pub fn is_quote_acct_for_margin_exch(
		&self,
		raw_accts: &HashMap<u16, RawAccount>,
		acct_map: &HashMap<u16, Account>
	) -> bool {

		let acct = acct_map.get(&self.account_key).unwrap();
		let raw_acct = raw_accts.get(&acct.raw_key).unwrap();
		raw_acct.ticker.contains('_')
	}

	pub fn get_mvmts_in_ar(
        &self,
        acct_map: &HashMap<u16, Account>,
        txns_map: &HashMap<u32, Transaction>,
    ) -> Vec<Rc<Movement>> {

        let polarity = Self::direction(self);
        let txn = txns_map.get(&self.tx_key).unwrap();
        let mut movements_in_ar = [].to_vec();
        let acct = acct_map.get(&self.account_key).unwrap();

        for lot in acct.list_of_lots.borrow().iter() {
            for mvmt in lot.movements.borrow().iter() {
                if (mvmt.date) <= txn.date {
                    if mvmt.action_record_key == self.self_ar_key {
                        // if polarity == Polarity::Incoming{
                            // movements_in_ar.push(mvmt.clone())
                        // } else {
                            movements_in_ar.insert(0, mvmt.clone())
                        // }
                        //  ^^ leaving that ugliness for this commit on purpose
                    }
                }
            }
        }
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
