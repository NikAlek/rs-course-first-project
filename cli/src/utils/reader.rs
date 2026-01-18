use lib::model::data::TxData;
use lib::model::data::Status;
use lib::model::data::Format;
use lib::model::data::TxType;
use lib::{
    console::commands::{Commands, Resource},
    model::{errors::ParserErr},
    parser::{bin_psrser::TxnFromBin, csv_parser::TxnFromCsv, text_parser::TxnFromText},
};
use std::fs::File;

use std::io::{self, BufReader, BufWriter, Read, Write, stdin, stdout};

pub fn read(resource: &Resource, format: &Format) -> Result<Vec<TxData>, ParserErr> {
    let reader: Box<dyn Read> = match resource {
        Resource::Console => Box::new(stdin()),
        Resource::File { path } => {
            let file = File::open(path).map_err(|e| ParserErr::ParseErr { msg: e.to_string() })?;
            Box::new(BufReader::new(file))
        }
    };

    read_from_resource(reader, format)
}

fn read_from_resource(resource: Box<dyn Read>, format: &Format) -> Result<Vec<TxData>, ParserErr> {
    return match format {
        Format::YpBankBin => TxData::from_bin_reader(resource),
        Format::YpBankCsv => TxData::from_csv_reader(resource),
        Format::YpBankText => TxData::from_text_reader(resource),
    };
}

