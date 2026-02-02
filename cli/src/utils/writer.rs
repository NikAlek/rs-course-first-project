use lib::console::commands::Resource;
use lib::model::data::Format;
use lib::model::data::TxData;
use lib::model::errors::ParserErr;
use lib::parser::bin_psrser::TxnToBin;
use lib::parser::csv_parser::TxnToCsv;
use lib::parser::text_parser::TxnToText;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write, stdin, stdout};

fn write(resource: &Resource) -> Result<Box<dyn Write>, ParserErr> {
    match resource {
        Resource::Console => Ok(Box::new(stdout())),
        Resource::File { path } => {
            let file =
                File::create(path).map_err(|e| ParserErr::ParseErr { msg: e.to_string() })?;
            Ok(Box::new(BufWriter::new(file)))
        }
    }
}

pub fn write_to_resource(
    txns: &[TxData],
    resource: &Resource,
    format: &Format,
) -> Result<(), ParserErr> {
    let mut output = write(resource)?;

    let data_to_write = match format {
        Format::YpBankBin => {
            let mut buffer = Vec::new();
            for txn in txns {
                let bytes = txn.to_bin()?;
                buffer.extend(bytes);
            }
            buffer
        }
        Format::YpBankCsv => {
            let mut lines = Vec::new();
            for txn in txns {
                let line = txn.to_csv()?;
                lines.push(line);
            }
            let content = lines.join("\n");
            content.into_bytes()
        }
        Format::YpBankText => {
            let mut lines = Vec::new();
            for txn in txns {
                let line = txn.to_text()?;
                lines.push(line);
            }
            let content = lines.join("\n");
            content.into_bytes()
        }
    };

    output.write_all(&data_to_write)
      .map_err(|e| ParserErr::ParseErr { msg: e.to_string() })?;

    output
        .flush()
        .map_err(|e| ParserErr::ParseErr { msg: e.to_string() })?;

    Ok(())
}
