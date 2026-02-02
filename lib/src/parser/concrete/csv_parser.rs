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

/// Трейт для парсинга транзакций из формата YbCSV.
///
/// Поддерживает три способа десериализации:
/// - одну строку YbCSV,
/// - коллекцию строк YbCSV,
/// - поток данных через `Read` (например, файл или stdin).
pub trait TxnFromCsv {
    /// Парсит одну транзакцию из строки в формате YbCSV.
    ///
    /// Ожидается, что строка содержит ровно одно YbCSV-запись (без символа новой строки в конце).
    ///
    /// # Errors
    /// Возвращает `ParserErr`, если строка некорректна, не соответствует ожидаемой схеме
    /// или содержит недопустимые значения.
    fn from_csv(csv_line: &str) -> Result<TxData, ParserErr>;

    /// Парсит несколько транзакций из набора YbCSV-строк.
    ///
    /// Каждая строка в `csv_lines` должна представлять отдельную запись.
    /// Обычно используется при предварительной загрузке данных в память.
    ///
    /// # Errors
    /// Возвращает `ParserErr`, если хотя бы одна из строк не может быть распарсена.
    fn from_csv_many(csv_lines: &[String]) -> Result<Vec<TxData>, ParserErr>;

    /// Парсит транзакции из потока данных, реализующего `Read`.
    ///
    /// Метод читает весь поток как YbCSV-документ (обычно построчно),
    /// пропуская заголовок, если он есть (это зависит от реализации).
    ///
    /// # Errors
    /// Возвращает `ParserErr`, если произошла ошибка чтения или парсинга любой записи.
    fn from_csv_reader(reader: Box<dyn Read>) -> Result<Vec<TxData>, ParserErr>;
}

/// Трейт для сериализации транзакций в формат YbCSV.
///
/// Предоставляет методы для преобразования одной или нескольких транзакций
/// в текстовое представление, совместимое с YbCSV.
pub trait TxnToCsv {
    /// Сериализует одну транзакцию в строку формата YbCSV.
    ///
    /// Результат не включает завершающий символ новой строки.
    ///
    /// # Errors
    /// Возвращает `ParserErr`, если сериализация невозможна
    /// (например, из-за отсутствующих обязательных полей или ошибки экранирования).
    fn to_csv(&self) -> Result<String, ParserErr>;

    /// Сериализует множество транзакций в единый YbCSV-документ.
    ///
    /// Обычно результат включает заголовок (если применимо) и каждую транзакцию на отдельной строке.
    /// Конкретный формат (наличие заголовка, порядок полей и т.д.) определяется реализацией.
    ///
    /// # Errors
    /// Возвращает `ParserErr`, если сериализация хотя бы одной транзакции завершилась неудачно.
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_from_csv_valid_line() {
        let line = "123,TRANSFER,456,789,100,1700000000,SUCCESS,\"Test transfer\"";
        let tx = TxData::from_csv(line).unwrap();

        assert_eq!(tx.tx_id, 123);
        assert_eq!(tx.tx_type, TxType::Transfer);
        assert_eq!(tx.from_user_id, 456);
        assert_eq!(tx.to_user_id, 789);
        assert_eq!(tx.amount, 100);
        assert_eq!(tx.timestamp, 1700000000);
        assert_eq!(tx.status, Status::Success);
        assert_eq!(tx.description, "Test transfer");
        assert_eq!(tx.format, Format::YpBankCsv);
    }

    #[test]
    fn test_from_csv_many_valid() {
        let lines = vec![
            CSV_HEADER_LINE.to_string(),
            "1000000000000011,WITHDRAWAL,9223372036854775807,0,1200,1633037520000,SUCCESS,\"Record number 12\"".to_string(),
            "1000000000000012,DEPOSIT,0,9223372036854775807,1300,1633037580000,FAILURE,\"Record number 13\"".to_string(),
        ];

        let txs = TxData::from_csv_many(&lines).unwrap();
        assert_eq!(txs.len(), 2);
        assert_eq!(txs[0].tx_type, TxType::Withdrawal);
        assert_eq!(txs[1].tx_type, TxType::Deposit);
    }

    #[test]
    fn test_from_csv_many_empty() {
        let lines: Vec<String> = vec![];
        let err = TxData::from_csv_many(&lines).unwrap_err();
        if let ParserErr::ParseErr { msg } = err {
            assert!(msg.contains("CSV is empty"));
        } else {
            panic!()
        }
    }

    #[test]
    fn test_from_csv_many_invalid_header() {
        let lines = vec![
            "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS".to_string(), // missing DESCRIPTION
        ];
        let err = TxData::from_csv_many(&lines).unwrap_err();
        if let ParserErr::ParseErr { msg } = err {
            assert!(msg.contains("Invalid header"));
        } else {
            panic!()
        }
    }

    #[test]
    fn test_from_csv_reader_valid() {
        let csv_content = vec![
            CSV_HEADER_LINE.to_string(),
            "1000000000000012,DEPOSIT,0,9223372036854775807,1300,1633037580000,FAILURE,\"Record number 13\"".to_string()
        ];

        let csv_content = csv_content.join("\n");
        let cursor: Cursor<Vec<u8>> = Cursor::new(csv_content.into_bytes());
        let reader: Box<dyn Read> = Box::new(cursor);

        let txs = TxData::from_csv_reader(reader).unwrap();
        assert_eq!(txs.len(), 1);
        assert_eq!(txs[0].tx_id, 1000000000000012);
        assert_eq!(txs[0].description, "Record number 13");
    }

    #[test]
    fn test_from_csv_reader_invalid_header_order() {
        let csv_content = "TX_ID,FROM_USER_ID,TX_TYPE,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n1,100,TRANSFER,200,50,1700000000,SUCCESS,\"ok\"";
        let reader = Box::new(Cursor::new(csv_content));
        let err = TxData::from_csv_reader(reader).unwrap_err();
        if let ParserErr::ParseErr { msg } = err {
            assert!(msg.contains("Invalid CSV header"));
        } else {
            panic!()
        }
    }

    #[test]
    fn test_to_csv_simple() {
        let tx = TxData {
            tx_id: 42,
            tx_type: TxType::Withdrawal,
            from_user_id: 101,
            to_user_id: 0,
            amount: 30,
            timestamp: 1700000010,
            status: Status::Success,
            description: "Cash out".to_string(),
            format: Format::YpBankCsv,
        };

        let csv = tx.to_csv().unwrap();
        assert_eq!(
            csv,
            "42,WITHDRAWAL,101,0,30,1700000010,SUCCESS,\"Cash out\""
        );
    }

    #[test]
    fn test_to_csv_description_needs_escaping() {
        let tx = TxData {
            tx_id: 99,
            tx_type: TxType::Transfer,
            from_user_id: 1,
            to_user_id: 2,
            amount: 10,
            timestamp: 1700000020,
            status: Status::Pending,
            description: "Amount: \"10\", note: comma, and\nnewline".to_string(),
            format: Format::YpBankCsv,
        };

        let csv = tx.to_csv().unwrap();
        assert_eq!(
            csv,
            "99,TRANSFER,1,2,10,1700000020,PENDING,\"Amount: \"\"10\"\", note: comma, and\nnewline\""
        );
    }

    #[test]
    fn test_to_csv_many() {
        let txs = vec![
            TxData {
                tx_id: 1,
                tx_type: TxType::Deposit,
                from_user_id: 0,
                to_user_id: 10,
                amount: 100,
                timestamp: 1700000030,
                status: Status::Success,
                description: "Bonus".to_string(),
                format: Format::YpBankCsv,
            },
            TxData {
                tx_id: 2,
                tx_type: TxType::Transfer,
                from_user_id: 10,
                to_user_id: 20,
                amount: 25,
                timestamp: 1700000040,
                status: Status::Failure,
                description: "Blocked".to_string(),
                format: Format::YpBankCsv,
            },
        ];

        let csv = TxData::to_csv_many(&txs).unwrap();
        let expected = format!(
            "{}\n1,DEPOSIT,0,10,100,1700000030,SUCCESS,\"Bonus\"\n2,TRANSFER,10,20,25,1700000040,FAILURE,\"Blocked\"\n",
            CSV_HEADER_LINE
        );
        assert_eq!(csv, expected);
    }

    #[test]
    fn test_escape_csv_field() {
        assert_eq!(escape_csv_field("plain"), "plain");
        assert_eq!(escape_csv_field("with,comma"), "with,comma");
        assert_eq!(escape_csv_field("with\nnewline"), "with\nnewline");
        assert_eq!(escape_csv_field(r#"say "hello""#), r#"say ""hello"""#);
        assert_eq!(
            escape_csv_field("mixed \"quotes\", commas, and\nnewlines"),
            "mixed \"\"quotes\"\", commas, and\nnewlines"
        );
    }
}
