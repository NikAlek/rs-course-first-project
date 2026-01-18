use core::error;
use std::io::Read;

use crate::utils::reader::{self, read};
use lib::model::data::TxData;
use lib::{
    console::commands::{Commands, Resource},
    model::{data::Format, errors::ParserErr},
    parser::{bin_psrser::TxnFromBin, csv_parser::TxnFromCsv, text_parser::TxnFromText},
};


#[derive(Clone, Debug)]
pub struct ComparerLogicResult {
    result: bool,
}

#[derive(Clone, Debug)]
pub enum CompareLogicErr {
    Prepare { err: ParserErr },
    Logic,
}

pub fn process_comparer_logic(
    first_from: &Resource,
    first_format: &Format,
    second_from: &Resource,
    second_format: &Format,
) -> Result<ComparerLogicResult, CompareLogicErr> {
    let first_txn =
        read(first_from, first_format).map_err(|err| CompareLogicErr::Prepare { err: err })?;
    let  second_txn =
        read(second_from, second_format).map_err(|err| CompareLogicErr::Prepare { err: err })?;


    if first_txn == second_txn {
        Ok(ComparerLogicResult { result: true })
    } else {
        Ok(ComparerLogicResult { result: false })
    }
}


