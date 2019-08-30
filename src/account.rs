// Copyright (c) 2017-2019, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools-rs/blob/master/LEGAL.txt

use std::rc::{Rc, Weak};
use std::cell::{Cell, RefCell};
use std::fmt;
use std::collections::{HashMap};
use std::error::Error;

use chrono::{NaiveDate, NaiveTime, NaiveDateTime, DateTime, Utc, TimeZone};
use chrono_tz::US::Eastern;
use decimal::d128;
use serde_derive::{Serialize, Deserialize};

use crate::transaction::{Transaction, ActionRecord, Polarity, TxType};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct RawAccount {
	pub account_num: u16,
	pub name: String,
	pub ticker: String,
	pub is_margin: bool,
}

impl RawAccount {
	pub fn is_home_currency(&self, compare: &String) -> bool {
		&self.ticker == compare
	}
}

#[derive(Clone, Debug)]
pub struct Account {
	pub raw_key: u16,
	pub list_of_lots: RefCell<Vec<Rc<Lot>>>,
	// pub vec_of_lot_keys: (RawAccount, u32),
}

impl Account {

	pub fn is_home_currency(&self, compare: &String, raw_acct_map: &HashMap<u16, RawAccount>) -> bool {
		let raw_acct = raw_acct_map.get(&self.raw_key).unwrap();
		&raw_acct.ticker == compare
	}

	pub fn get_sum_of_amts_in_lots(&self) -> d128 {
		let lots = self.list_of_lots.borrow();
		let mut total_amount = d128!(0);
			for lot in lots.iter() {
				let sum = lot.get_sum_of_amts_in_lot();
				total_amount += sum;
			}
		total_amount
	}

	pub fn get_sum_of_basis_in_lots(&self) -> d128 {
		let lots = self.list_of_lots.borrow();
		let mut total_amount = d128!(0);
			for lot in lots.iter() {
				let sum = lot.get_sum_of_basis_in_lot();
				total_amount += sum;
			}
		total_amount
	}
}

#[derive(Clone, Debug)]
pub struct RawMarginPair (pub Weak<RawAccount>, pub Weak<RawAccount>);	    //  always (base_acct, quote_acct)

#[derive(Clone, Debug)]
pub struct Lot {
	pub date_as_string: String,
	pub date_of_first_mvmt_in_lot: NaiveDate,
	pub date_for_basis_purposes: NaiveDate,
	pub lot_number: u32,	//	Does NOT start at zero.  First lot is lot 1.
	pub account_key: u16,
	pub movements: RefCell<Vec<Rc<Movement>>>,
}

impl Lot {
	pub fn get_sum_of_amts_in_lot(&self) -> d128 {
		let mut amts = d128!(0);
		self.movements.borrow().iter().for_each(|movement| amts += movement.amount);
		amts
	}
	pub fn sum_of_amts_in_lot_is_zero(&self) -> bool {
		d128!(0) == Self::get_sum_of_amts_in_lot(&self)
	}
	pub fn get_sum_of_basis_in_lot(&self) -> d128 {
		let mut amts = d128!(0);
		self.movements.borrow().iter().for_each(|movement| amts += movement.cost_basis.get());
		amts
	}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Movement {
	pub amount: d128,
	pub date_as_string: String,
	pub date: NaiveDate,
	pub transaction_key: u32,
	pub action_record_key: u32,
	pub cost_basis: Cell<d128>,	//	Initialized with 0. Set in add_cost_basis_to_movements()
	pub ratio_of_amt_to_incoming_mvmts_in_a_r: d128,	//	Set in process_multiple_incoming_lots_and_mvmts() and incoming flow dual actionrecord transactions
	pub ratio_of_amt_to_outgoing_mvmts_in_a_r: Cell<d128>,	//	Set in wrap_mvmt_and_push()
	pub lot_num: u32,
	pub proceeds: Cell<d128>,	//	Initialized with 0. Set in add_proceeds_to_movements()
}

impl Movement {

	pub fn get_lot(
        &self,
        acct_map: &HashMap<u16, Account>,
        ar_map: &HashMap<u32, ActionRecord>
    ) -> Rc<Lot> {
		let ar = ar_map.get(&self.action_record_key).unwrap();
		let acct = acct_map.get(&ar.account_key).unwrap();
		let lot = acct.list_of_lots.borrow()[self.lot_num as usize - 1].clone();	//	lots start at 1 and indexes at 0
		lot
	}

	pub fn ratio_of_amt_to_lots_first_mvmt(
        &self,
        acct_map: &HashMap<u16, Account>,
        ar_map: &HashMap<u32, ActionRecord>
    ) -> d128 {
		// println!("Lot #: {}", self.lot.lot_number);
		let ar = ar_map.get(&self.action_record_key).unwrap();
		let acct = acct_map.get(&ar.account_key).unwrap();
		let lot = &acct.list_of_lots.borrow()[self.lot_num as usize - 1];	//	lots start at 1 and indexes at 0
		let borrowed_mvmt_list = lot.movements.borrow();
		let ratio = self.amount / borrowed_mvmt_list.first().unwrap().amount;
		// println!("ratio_of_amt_to_lots_first_mvmt: {}", ratio.abs());
		ratio.abs()
	}

	pub fn get_gain_or_loss(&self) -> d128 {
		self.proceeds.get() + self.cost_basis.get()
	}

	pub fn get_term(&self, acct_map: &HashMap<u16, Account>, ar_map: &HashMap<u32, ActionRecord>,) -> Term {

    	use time::Duration;
		let ar = ar_map.get(&self.action_record_key).unwrap();
		let lot = Self::get_lot(&self, acct_map, ar_map);

    	match ar.direction() {
			Polarity::Incoming => {
				let today = Utc::now();
				let utc_lot_date = Self::create_date_time_from_atlantic(lot.date_for_basis_purposes, NaiveTime::from_hms_milli(12, 34, 56, 789));
				// if today.signed_duration_since(self.lot.date_for_basis_purposes) > 365
				if (today - utc_lot_date) > Duration::days(365) {	//	TODO: figure out how to instantiate today's date and convert it to compare to NaiveDate
					Term::LT
				}
				else {
					Term::ST
				}
			}
			Polarity::Outgoing => {
				let lot_date_for_basis_purposes = lot.date_for_basis_purposes;
				if self.date.signed_duration_since(lot_date_for_basis_purposes) > Duration::days(365) {
					return Term::LT
				}
				Term::ST
			}
		}
	}

	pub fn create_date_time_from_atlantic(date: NaiveDate, time: NaiveTime) -> DateTime<Utc> {
		let naive_datetime = NaiveDateTime::new(date, time);
		let east_time = Eastern.from_local_datetime(&naive_datetime).unwrap();
		east_time.with_timezone(&Utc)
	}

	pub fn get_income(
		&self,
		ar_map: &HashMap<u32,
		ActionRecord>,
		raw_accts: &HashMap<u16, RawAccount>,
		acct_map: &HashMap<u16, Account>,
		txns_map: &HashMap<u32, Transaction>,
	)-> Result<d128, Box<dyn Error>> {

		let txn = txns_map.get(&self.transaction_key).expect("Couldn't get txn. Tx num invalid?");
		match txn.transaction_type(ar_map, raw_accts, acct_map)? {
			TxType::Flow => {
				let ar = ar_map.get(&self.action_record_key).unwrap();
				if ar.direction() == Polarity::Incoming {
					Ok(self.proceeds.get())
				}
				else { Ok(d128!(0)) }
			}
			TxType::Exchange => { Ok(d128!(0)) }
			TxType::ToSelf => { Ok(d128!(0)) }
		}
	}

	pub fn get_expense(
		&self,
		ar_map: &HashMap<u32, ActionRecord>,
		raw_accts: &HashMap<u16, RawAccount>,
		acct_map: &HashMap<u16, Account>,
		txns_map: &HashMap<u32, Transaction>,
	)-> Result<d128, Box<dyn Error>> {

		let txn = txns_map.get(&self.transaction_key).expect("Couldn't get txn. Tx num invalid?");
		match txn.transaction_type(ar_map, raw_accts, acct_map)? {
			TxType::Flow => {
				let ar = ar_map.get(&self.action_record_key).unwrap();
				if ar.direction() == Polarity::Outgoing {
					Ok(self.proceeds.get())
				}
				else { Ok(d128!(0)) }
			}
			TxType::Exchange => { Ok(d128!(0)) }
			TxType::ToSelf => { Ok(d128!(0)) }
		}
	}

    pub fn direction(&self) -> Polarity {
		if self.amount < d128!(0.0) { Polarity::Outgoing }
		else { Polarity::Incoming }
	}
}

#[derive(Clone, Debug, PartialEq)]
pub enum Term {
	LT,
	ST,
}

impl Term {

    pub fn abbr_string(&self) -> String {
        match *self {
            Term::LT => "LT".to_string(),
            Term::ST => "ST".to_string(),
        }
    }
}

impl fmt::Display for Term {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       match *self {
           Term::LT => write!(f, "LT"),
           Term::ST => write!(f, "ST"),
       }
    }
}
