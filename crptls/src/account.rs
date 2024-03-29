// Copyright (c) 2017-2023, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools/blob/master/LEGAL.txt

use std::rc::{Rc, Weak};
use std::cell::{Cell, RefCell};
use std::fmt;
use std::collections::HashMap;
use std::error::Error;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
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
	pub fn is_home_currency(&self, compare: &str) -> bool {
		self.ticker == compare
    }

    pub fn margin_string(&self) -> String {
        if self.is_margin {
            "Margin".to_string()
        } else {
            "Non-margin".to_string()
        }
    }
}

#[derive(Clone, Debug)]
pub struct Account {
	pub raw_key: u16,
	pub list_of_lots: RefCell<Vec<Rc<Lot>>>,
	// pub vec_of_lot_keys: (RawAccount, u32),
}

impl Account {

	pub fn get_sum_of_amts_in_lots(&self) -> Decimal {
		let lots = self.list_of_lots.borrow();
		let mut total_amount = dec!(0);
			for lot in lots.iter() {
				let sum = lot.get_sum_of_amts_in_lot();
				total_amount += sum;
			}
		total_amount
	}

	pub fn get_sum_of_lk_basis_in_lots(&self) -> Decimal {
		let lots = self.list_of_lots.borrow();
		let mut total_amount = dec!(0);
			for lot in lots.iter() {
				let sum = lot.get_sum_of_lk_basis_in_lot();
				total_amount += sum;
			}
		total_amount
	}

	pub fn get_sum_of_orig_basis_in_lots(&self) -> Decimal {
		let lots = self.list_of_lots.borrow();
		let mut total_amount = dec!(0);
			for lot in lots.iter() {
				let sum = lot.get_sum_of_orig_basis_in_lot();
				total_amount += sum;
			}
		total_amount
    }

    pub fn get_num_of_nonzero_lots(&self) -> u32 {

        let mut count = 0;

        for lot in self.list_of_lots.borrow().iter() {
            if lot.get_sum_of_amts_in_lot() > dec!(0) {
                count += 1
            }
        }

        count
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
	pub fn get_sum_of_amts_in_lot(&self) -> Decimal {
		let mut amts = dec!(0);
		self.movements.borrow().iter().for_each(|movement| amts += movement.amount);
		amts
	}

	pub fn get_sum_of_lk_basis_in_lot(&self) -> Decimal {
		let mut amts = dec!(0);
		self.movements.borrow().iter().for_each(|movement| amts += movement.cost_basis_lk.get());
		amts
	}

	pub fn get_sum_of_orig_basis_in_lot(&self) -> Decimal {
		let mut amts = dec!(0);
		self.movements.borrow().iter().for_each(|movement| amts += movement.cost_basis.get());
		amts
	}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Movement {
	pub amount: Decimal,
	pub date_as_string: String,
	pub date: NaiveDate,
	pub transaction_key: u32,
	pub action_record_key: u32,
	pub cost_basis: Cell<Decimal>,	//	Initialized with 0. Set in add_cost_basis_to_movements()
	pub ratio_of_amt_to_incoming_mvmts_in_a_r: Decimal,	//	Set in process_multiple_incoming_lots_and_mvmts() and incoming flow dual actionrecord transactions
	pub ratio_of_amt_to_outgoing_mvmts_in_a_r: Cell<Decimal>,	//	Set in wrap_mvmt_and_push()
	pub lot_num: u32,
	pub proceeds: Cell<Decimal>,	//	Initialized with 0. Set in add_proceeds_to_movements()
    pub proceeds_lk: Cell<Decimal>,
    pub cost_basis_lk: Cell<Decimal>,
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
    ) -> Decimal {

        let lot = self.get_lot(acct_map, ar_map);
		let list_of_lot_mvmts = lot.movements.borrow();
		let ratio = self.amount / list_of_lot_mvmts.first().unwrap().amount;

		ratio.abs()
	}

    pub fn get_lk_cost_basis_of_lots_first_mvmt(
        &self,
        acct_map: &HashMap<u16, Account>,
        ar_map: &HashMap<u32, ActionRecord>
    ) -> Decimal {

        let lot = self.get_lot(acct_map, ar_map);
		let list_of_lot_mvmts = lot.movements.borrow();
		let cost_basis_lk = list_of_lot_mvmts.first().unwrap().cost_basis_lk.get();

		cost_basis_lk
	}

    pub fn get_cost_basis_of_lots_first_mvmt(
        &self,
        acct_map: &HashMap<u16, Account>,
        ar_map: &HashMap<u32, ActionRecord>
    ) -> Decimal {

        let lot = self.get_lot(acct_map, ar_map);
		let list_of_lot_mvmts = lot.movements.borrow();
		let cost_basis = list_of_lot_mvmts.first().unwrap().cost_basis.get();

		cost_basis
	}

	pub fn get_lk_gain_or_loss(&self) -> Decimal {
		self.proceeds_lk.get() + self.cost_basis_lk.get()
	}

    pub fn get_orig_gain_or_loss(&self) -> Decimal {
		self.proceeds.get() + self.cost_basis.get()
	}

	/// This function is only called during export operations.  In addition, this will
	/// only be called on flow and outgoing exchange `transactions`. Lastly, the only
	/// `movement`s subject to this call with have non-margin accounts.
	pub fn get_term(
		&self,
		acct_map: &HashMap<u16, Account>,
		ar_map: &HashMap<u32, ActionRecord>,
		txns_map: &HashMap<u32, Transaction>
	) -> Term {

		let ar = ar_map.get(&self.action_record_key).unwrap();
		let lot = Self::get_lot(&self, acct_map, ar_map);

    	match ar.direction() {

			Polarity::Incoming => {

				// For a dual-`action record` `transaction` with a non-margin `account` incoming amount,
				// if there was like-kind treatment, the basis date may be before the `transaction` date.
				let txn = txns_map.get(&self.transaction_key).unwrap();
				if txn.action_record_idx_vec.len() == 2 {
					let lot_date_for_basis_purposes = lot.date_for_basis_purposes;
					if self.date.signed_duration_since(lot_date_for_basis_purposes) > chrono::Duration::days(365) {
						return Term::LT
					}
					return Term::ST
				}

				// For a single-`action record` `transaction`, term is meaningless, but it is being shown
				// in the context of the holding period, in the event it were sold "today".
				let today: NaiveDate = chrono::Local::now().naive_utc().date();
				if today.signed_duration_since(lot.date_for_basis_purposes) > chrono::Duration::days(365) {
					Term::LT
				}
				else {
					Term::ST
				}
			}

			Polarity::Outgoing => {

				let lot_date_for_basis_purposes = lot.date_for_basis_purposes;

                if self.date.signed_duration_since(lot_date_for_basis_purposes) > chrono::Duration::days(365) {
					return Term::LT
				}
				Term::ST
			}
		}
	}

	pub fn get_income(
		&self,
		ar_map: &HashMap<u32,
		ActionRecord>,
		raw_accts: &HashMap<u16, RawAccount>,
		acct_map: &HashMap<u16, Account>,
		txns_map: &HashMap<u32, Transaction>,
	)-> Result<Decimal, Box<dyn Error>> {  //  Returns 0 or positive number

		let txn = txns_map.get(&self.transaction_key).expect("Couldn't get txn. Tx num invalid?");

		match txn.transaction_type(ar_map, raw_accts, acct_map)? {

			TxType::Flow => {

				let ar = ar_map.get(&self.action_record_key).unwrap();

                if ar.direction() == Polarity::Incoming {
					Ok(-self.proceeds_lk.get())
				}
				else { Ok(dec!(0)) }
			}
			TxType::Exchange => { Ok(dec!(0)) }
			TxType::ToSelf => { Ok(dec!(0)) }
		}
	}

	pub fn get_expense(
		&self,
		ar_map: &HashMap<u32, ActionRecord>,
		raw_accts: &HashMap<u16, RawAccount>,
		acct_map: &HashMap<u16, Account>,
		txns_map: &HashMap<u32, Transaction>,
	)-> Result<Decimal, Box<dyn Error>> {  //  Returns 0 or negative number

		let txn = txns_map.get(&self.transaction_key).expect("Couldn't get txn. Tx num invalid?");

		match txn.transaction_type(ar_map, raw_accts, acct_map)? {

			TxType::Flow => {

				let ar = ar_map.get(&self.action_record_key).unwrap();

                if ar.direction() == Polarity::Outgoing {

                    let acct = acct_map.get(&ar.account_key).unwrap();
                    let raw_acct = raw_accts.get(&acct.raw_key).unwrap();

                    if raw_acct.is_margin {

                       Ok(dec!(0))

                    } else {

                        let expense = -self.proceeds_lk.get();
                        Ok(expense)
                    }
				}
				else { Ok(dec!(0)) }
			}
			TxType::Exchange => { Ok(dec!(0)) }
			TxType::ToSelf => { Ok(dec!(0)) }
		}
    }

    pub fn friendly_tx_type(&self, tx_type: &TxType) -> String {

        let tx_type_string = match tx_type {

            TxType::Exchange => { tx_type.to_string() },

            TxType::ToSelf => { tx_type.to_string() },

            TxType::Flow => {

                let direction: String;

                if self.amount > dec!(0) {
                    direction = "In".to_string();
                } else {
                    direction = "Out".to_string()
                }

                direction + &tx_type.to_string().to_lowercase()
            },
        };

        tx_type_string
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
