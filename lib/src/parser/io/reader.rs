use std::fs::File;

use std::io::{self, BufReader, BufWriter, Read, Write, stdin, stdout};

use crate::console::commands::Resource;
use crate::model::data::{Format, TxData};
use crate::model::errors::ParserErr;
use crate::parser::concrete::bin_psrser::TxnFromBin;
use crate::parser::concrete::csv_parser::TxnFromCsv;
use crate::parser::concrete::text_parser::TxnFromText;

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

