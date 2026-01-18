use csv::{ReaderBuilder, StringRecord};
use std::io::{Cursor, Read};

use crate::model::data::Format;
use crate::model::data::Status;
use crate::model::data::TxData;
use crate::model::data::TxType;
use crate::model::errors::ParserErr;

const CSV_HEADERS: &[&str] = &[
    "TX_ID",
    "TX_TYPE",
    "FROM_USER_ID",
    "TO_USER_ID",
    "AMOUNT",
    "TIMESTAMP",
    "STATUS",
    "DESCRIPTION",
];

const CSV_HEADER_LINE: &str =
    "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION";

pub trait TxnFromCsv {
    fn from_csv(csv_line: &str) -> Result<TxData, ParserErr>;
    fn from_csv_many(csv_lines: &[String]) -> Result<Vec<TxData>, ParserErr>;
    fn from_csv_reader(reader: Box<dyn Read>) -> Result<Vec<TxData>, ParserErr>;
}

pub trait TxnToCsv {
    fn to_csv(&self) -> Result<String, ParserErr>;
    fn to_csv_many(many: &[Self]) -> Result<String, ParserErr>
    where
        Self: Sized;
}

impl TxnFromCsv for TxData {
    fn from_csv(csv_line: &str) -> Result<TxData, ParserErr> {
        let mut rdr = ReaderBuilder::new()
            .has_headers(false)
            .delimiter(b',')
            .from_reader(Cursor::new(csv_line.as_bytes()));

        let record = rdr
            .records()
            .next()
            .ok_or_else(|| ParserErr::ParseErr {
                msg: "Empty CSV line".into(),
            })?
            .map_err(|e| ParserErr::ParseErr { msg: e.to_string() })?;

        from_csv_record(&record)
    }

    fn from_csv_many(csv_lines: &[String]) -> Result<Vec<TxData>, ParserErr> {
        // Первая строка — заголовок
        if csv_lines.is_empty() {
            return Err(ParserErr::ParseErr {
                msg: "CSV is empty".into(),
            });
        }

        let header = &csv_lines[0];

        if header.trim() != CSV_HEADER_LINE {
            return Err(ParserErr::ParseErr {
                msg: format!(
                    "Invalid header: expected '{}', got '{}'",
                    CSV_HEADER_LINE, header
                ),
            });
        }

        let mut transactions = Vec::with_capacity(csv_lines.len() - 1);
        for (i, line) in csv_lines.iter().enumerate().skip(1) {
            if line.trim().is_empty() {
                continue;
            }
            match Self::from_csv(line) {
                Ok(tx) => transactions.push(tx),
                Err(e) => {
                    return Err(ParserErr::ParseErr {
                        msg: format!("Error parsing line {}: {}", i + 1, e),
                    });
                }
            }
        }
        Ok(transactions)
    }

    fn from_csv_reader(reader: Box<dyn Read>) -> Result<Vec<TxData>, ParserErr> {
        let mut rdr = ReaderBuilder::new()
            .has_headers(true)
            .delimiter(b',')
            .from_reader(reader);

        let actual_headers = rdr
            .headers()
            .map_err(|e| ParserErr::ParseErr {
                msg: format!("Failed to read CSV header: {}", e),
            })?
            .iter()
            .collect::<Vec<_>>();

        if actual_headers != CSV_HEADERS {
            return Err(ParserErr::ParseErr {
                msg: format!(
                    "Invalid CSV header. Expected: {:?}, got: {:?}",
                    CSV_HEADERS, actual_headers
                ),
            });
        }

        let mut transactions = Vec::new();
        for (i, result) in rdr.records().enumerate() {
            let record = result.map_err(|e| ParserErr::ParseErr {
                msg: format!("CSV parse error on row {}: {}", i + 2, e),
            })?;
            if record.iter().all(|f| f.is_empty()) {
                continue;
            }
            let tx = from_csv_record(&record).map_err(|e| ParserErr::ParseErr {
                msg: format!("Field error on row {}: {}", i + 2, e),
            })?;
            transactions.push(tx);
        }
        Ok(transactions)
    }
}

fn from_csv_record(record: &StringRecord) -> Result<TxData, ParserErr> {
    if record.len() != 8 {
        return Err(ParserErr::ParseErr {
            msg: format!("Expected 8 fields, got {}", record.len()),
        });
    }

    let tx_id = record[0].parse().map_err(|_| ParserErr::ParseErr {
        msg: "Invalid TX_ID".into(),
    })?;
    let tx_type = parse_tx_type_str(&record[1])?;
    let from_user_id = record[2].parse().map_err(|_| ParserErr::ParseErr {
        msg: "Invalid from_user_id".into(),
    })?;
    let to_user_id = record[3].parse().map_err(|_| ParserErr::ParseErr {
        msg: "Invalid to_user_id".into(),
    })?;
    let amount = record[4].parse().map_err(|_| ParserErr::ParseErr {
        msg: "Invalid amount".into(),
    })?;
    let timestamp = record[5].parse().map_err(|_| ParserErr::ParseErr {
        msg: "Invalid TIMESTAMP".into(),
    })?;
    let status = parse_status_str(&record[6])?;
    let description = record[7].to_string();

    Ok(TxData {
        tx_id: tx_id,
        tx_type: tx_type,
        from_user_id: from_user_id,
        to_user_id: to_user_id,
        amount: amount,
        timestamp: timestamp,
        status: status,
        description: description,
        format: Format::YpBankCsv,
    })
}

fn parse_tx_type_str(s: &str) -> Result<TxType, ParserErr> {
    //TODO заилайнить
    match s {
        "DEPOSIT" => Ok(TxType::Deposit),
        "TRANSFER" => Ok(TxType::Transfer),
        "WITHDRAWAL" => Ok(TxType::Withdrawal),
        _ => Err(ParserErr::ParseErr {
            msg: format!("Invalid TX_TYPE: {}", s),
        }),
    }
}

fn parse_status_str(s: &str) -> Result<Status, ParserErr> {
    match s {
        "SUCCESS" => Ok(Status::Success),
        "FAILURE" => Ok(Status::Failure),
        "PENDING" => Ok(Status::Pending),
        _ => Err(ParserErr::ParseErr {
            msg: format!("Invalid STATUS: {}", s),
        }),
    }
}

impl TxnToCsv for TxData {
    fn to_csv(&self) -> Result<String, ParserErr> {
        let tx_type_str = match self.tx_type {
            TxType::Deposit => "DEPOSIT",
            TxType::Transfer => "TRANSFER",
            TxType::Withdrawal => "WITHDRAWAL",
            _ => {
                return Err(ParserErr::ParseErr {
                    msg: format!("Invalid TYPE: {:?}", &self.tx_type),
                });
            }
        };

        let status_str = match self.status {
            Status::Success => "SUCCESS",
            Status::Failure => "FAILURE",
            Status::Pending => "PENDING",
            _ => {
                return Err(ParserErr::ParseErr {
                    msg: format!("Invalid STATUS: {:?}", &self.tx_type),
                });
            }
        };

        // Описание всегда в двойных кавычках (как в спецификации)
        let desc_escaped = escape_csv_field(&self.description);
        let desc_quoted = format!("\"{}\"", desc_escaped);

        Ok(format!(
            "{},{},{},{},{},{},{},{}",
            self.tx_id,
            tx_type_str,
            self.from_user_id,
            self.to_user_id,
            self.amount,
            self.timestamp,
            status_str,
            desc_quoted
        ))
    }

    fn to_csv_many(transactions: &[Self]) -> Result<String, ParserErr> {
        let mut output = String::from(CSV_HEADER_LINE);
        output.push('\n');
        for tx in transactions {
            output.push_str(&tx.to_csv()?);
            output.push('\n');
        }
        Ok(output)
    }
}

fn escape_csv_field(s: &str) -> String {
    if s.contains('"') || s.contains(',') || s.contains('\n') {
        let escaped = s.replace('"', "\"\"");
        escaped
    } else {
        s.to_string()
    }
}
