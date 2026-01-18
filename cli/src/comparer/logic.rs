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
    let mut first_resource =
        read(first_from).map_err(|err| CompareLogicErr::Prepare { err: err })?;
    let mut second_resource =
        read(second_from).map_err(|err| CompareLogicErr::Prepare { err: err })?;

    let txn1 = read_from_resource(first_format, first_resource)
        .map_err(|err| CompareLogicErr::Prepare { err: err })?;
    let txn2 = read_from_resource(second_format, second_resource)
        .map_err(|err| CompareLogicErr::Prepare { err: err })?;

    if txn1 == txn2 {
        Ok(ComparerLogicResult { result: true })
    } else {
        Ok(ComparerLogicResult { result: false })
    }
}

fn read_from_resource(format: &Format, resource: Box<dyn Read>) -> Result<Vec<TxData>, ParserErr> {
    return match format {
        Format::YpBankBin => TxData::from_bin_reader(resource),
        Format::YpBankCsv => TxData::from_csv_reader(resource),
        Format::YpBankText => TxData::from_text_reader(resource),
    };
}
