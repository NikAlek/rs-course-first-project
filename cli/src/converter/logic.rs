use core::error;
use std::io::Read;

use crate::utils::writer::write_to_resource;
use crate::utils::reader::read;
use lib::model::data::TxData;
use lib::{
    console::commands::{Commands, Resource},
    model::{data::Format, errors::ParserErr},
    parser::{bin_psrser::TxnFromBin, csv_parser::TxnFromCsv, text_parser::TxnFromText},
};


#[derive(Clone, Debug)]
pub struct ConvertLogicResult {
    success: bool,
}


#[derive(Clone, Debug)]
pub enum ConvertLogicErr {
    Prepare { err: ParserErr },
    Logic,
}

pub fn process_convert_logic(
    from: &Resource,
    from_format: &Format,
    to: &Resource,
    to_format: &Format,
) -> Result<ConvertLogicResult, ConvertLogicErr> {
    let data = read(from, from_format)
        .map_err(|err| ConvertLogicErr::Prepare { err })?;

    write_to_resource(&data, to, to_format)
       .map_err(|err| ConvertLogicErr::Prepare { err })?;

    Ok(ConvertLogicResult { success: true })
}