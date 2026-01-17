use std::collections::HashMap;

use csv::{ReaderBuilder, StringRecord};
use std::io::{Cursor, Read};

use crate::model::data::Format;
use crate::model::data::Status;
use crate::model::data::TxData;
use crate::model::data::TxType;
use crate::model::errors::ParserErr;

pub trait TxnFromText {
    fn from_text(kv: &HashMap<String, String>) -> Result<TxData, ParserErr>;
    fn from_text_many(lines: &[String]) -> Result<Vec<TxData>, ParserErr>; //стоит ли тут TxData изменить на Self и сделать where ?
    fn from_text_reader(reader: &mut dyn Read) -> Result<Vec<TxData>, ParserErr>;
}

pub trait TxnToText {
    fn to_text(&self) -> Result<String, ParserErr>;
    fn to_text_many(many: &[Self]) -> Result<String, ParserErr>
    where
        Self: Sized;
}

impl TxnFromText for TxData {
    fn from_text(fields: &HashMap<String, String>) -> Result<TxData, ParserErr> {
        let get = |key: &str| {
            fields.get(key).ok_or_else(|| ParserErr::ParseErr {
                msg: format!("Missing field: {}", key),
            })
        };

        let unquote = |s: &str| {
            if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
                s[1..s.len() - 1].to_string()
            } else {
                s.to_string()
            }
        };

        Ok(TxData {
            tx_id: get("TX_ID")?.parse().map_err(|_| ParserErr::ParseErr {
                msg: "Invalid TX_ID".into(),
            })?,
            tx_type: parse_tx_type_str(get("TX_TYPE")?)?,
            from_user_id: get("FROM_USER_ID")?
                .parse()
                .map_err(|_| ParserErr::ParseErr {
                    msg: "Invalid FROM_USER_ID".into(),
                })?,
            to_user_id: get("TO_USER_ID")?
                .parse()
                .map_err(|_| ParserErr::ParseErr {
                    msg: "Invalid TO_USER_ID".into(),
                })?,
            amount: get("AMOUNT")?.parse().map_err(|_| ParserErr::ParseErr {
                msg: "Invalid AMOUNT".into(),
            })?,
            timestamp: get("TIMESTAMP")?.parse().map_err(|_| ParserErr::ParseErr {
                msg: "Invalid TIMESTAMP".into(),
            })?,
            status: parse_status_str(get("STATUS")?)?,
            description: unquote(get("DESCRIPTION")?),
            format: Format::YpBankText,
        })
    }

    fn from_text_many(lines: &[String]) -> Result<Vec<TxData>, ParserErr> {
        let mut transactions = Vec::new();
        let mut current = HashMap::new();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                if !current.is_empty() {
                    transactions.push(Self::from_text(&current)?);
                    current.clear();
                }
                continue;
            }

            if let Some(pos) = trimmed.find(':') {
                let key = trimmed[..pos].trim().to_string();
                let value = trimmed[pos + 1..].trim().to_string();
                current.insert(key, value);
            } else {
                return Err(ParserErr::ParseErr {
                    msg: format!("Invalid key-value on line {}: {}", i + 1, line),
                });
            }
        }

        if !current.is_empty() {
            transactions.push(Self::from_text(&current)?);
        }

        Ok(transactions)
    }

    fn from_text_reader(reader: &mut dyn Read) -> Result<Vec<Self>, ParserErr> {
        let content = std::io::read_to_string(reader)
            .map_err(|e| ParserErr::ParseErr { msg: e.to_string() })?;
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        Self::from_text_many(&lines)
    }
}

fn parse_tx_type_str(s: &str) -> Result<TxType, ParserErr> {
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

impl TxnToText for TxData {
    fn to_text(&self) -> Result<String, ParserErr> {
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
                    msg: format!("Invalid TYPE: {:?}", &self.tx_type),
                });
            }
        };

        // Описание в двойных кавычках
        let desc_quoted = format!("\"{}\"", self.description);

        Ok(format!(
            "TX_ID: {}\n\
             TX_TYPE: {}\n\
             FROM_USER_ID: {}\n\
             TO_USER_ID: {}\n\
             AMOUNT: {}\n\
             TIMESTAMP: {}\n\
             STATUS: {}\n\
             DESCRIPTION: {}",
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

    fn to_text_many(transactions: &[Self]) -> Result<String, ParserErr> {
        let mut output = String::new();
        for (i, tx) in transactions.iter().enumerate() {
            if i > 0 {
                output.push('\n'); // пустая строка между записями
            }
            output.push_str(&tx.to_text()?);
        }
        Ok(output)
    }
}
