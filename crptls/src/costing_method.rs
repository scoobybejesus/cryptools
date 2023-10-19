// Copyright (c) 2017-2023, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools/blob/master/LEGAL.txt

use std::fmt;

/// An `InventoryMethod` determines the order in which a `Lot` is chosen when posting
/// `ActionRecord` amounts as individual `Movement`s.
#[derive(Clone, Debug, PartialEq)]
pub enum InventoryCostingMethod {
    /// 1. LIFO according to the order the lot was created.
    LIFObyLotCreationDate,
    /// 2. LIFO according to the basis date of the lot.
    LIFObyLotBasisDate,
    /// 3. FIFO according to the order the lot was created.
    FIFObyLotCreationDate,
    /// 4. FIFO according to the basis date of the lot.
    FIFObyLotBasisDate,
}

impl fmt::Display for InventoryCostingMethod {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
           InventoryCostingMethod::LIFObyLotCreationDate => write!(f, "LIFO by lot creation date"),
           InventoryCostingMethod::LIFObyLotBasisDate => write!(f, "LIFO by lot basis date"),
           InventoryCostingMethod::FIFObyLotCreationDate => write!(f, "FIFO by lot creation date"),
           InventoryCostingMethod::FIFObyLotBasisDate => write!(f, "FIFO by lot basis date"),
       }
    }
}