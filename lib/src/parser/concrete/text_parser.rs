use std::collections::HashMap;

use csv::{ReaderBuilder, StringRecord};
use std::io::{Cursor, Read};

use crate::model::data::Format;
use crate::model::data::Status;
use crate::model::data::TxData;
use crate::model::data::TxType;
use crate::model::errors::ParserErr;

/// Трейт для парсинга транзакций из текстового представления в виде пар "ключ–значение".
///
/// Все методы возвращают значения типа [`TxData`], что предполагает наличие единого
/// унифицированного типа транзакции в системе. Если вы планируете поддерживать
/// несколько разных типов транзакций, возможно, стоит обобщить трейт через `Self`.
pub trait TxnFromText {
    /// Создаёт транзакцию из карты ключ-значение, представляющей одну запись.
    ///
    /// Каждая пара в `kv` соответствует одному полю транзакции (например, "amount", "timestamp").
    ///
    /// # Errors
    /// Возвращает [`ParserErr`], если обязательные поля отсутствуют,
    /// значения имеют неверный формат или нарушают бизнес-логику.
    fn from_text(kv: &HashMap<String, String>) -> Result<TxData, ParserErr>;

    /// Парсит несколько транзакций из списка строк.
    ///
    /// Каждая строка в `lines` должна быть преобразована во внутреннее представление
    /// (например, разобрана как key=value,key2=value2 и т.д.), после чего передана
    /// в `from_text`. Конкретный формат строки определяется реализацией.
    ///
    /// # Errors
    /// Возвращает [`ParserErr`], если хотя бы одна строка не может быть корректно распарсена.
    fn from_text_many(lines: &[String]) -> Result<Vec<TxData>, ParserErr>;

    /// Парсит транзакции из потока данных (`Read`), такого как файл или stdin.
    ///
    /// Поток читается построчно; каждая строка интерпретируется как отдельная запись.
    /// Предполагается, что строки уже находятся в пригодном для парсинга текстовом формате.
    ///
    /// # Errors
    /// Возвращает [`ParserErr`] при ошибке чтения или при невозможности распарсить любую из строк.
    fn from_text_reader(reader: Box<dyn Read>) -> Result<Vec<TxData>, ParserErr>;
}

/// Трейт для сериализации транзакций в человекочитаемый текстовый формат.
///
/// Обычно используется для логирования, отладки или экспорта в простой текстовый формат.
pub trait TxnToText {
    /// Преобразует одну транзакцию в текстовое представление.
    ///
    /// Формат может быть произвольным (например, `key1=value1; key2=value2`),
    /// но должен быть согласован с ожиданиями парсера `TxnFromText`.
    ///
    /// # Errors
    /// Возвращает [`ParserErr`], если сериализация невозможна
    /// (например, из-за недопустимых значений или внутренней ошибки форматирования).
    fn to_text(&self) -> Result<String, ParserErr>;

    /// Сериализует множество транзакций в единый текстовый документ.
    ///
    /// Обычно каждая транзакция представляется на отдельной строке.
    /// Разделитель строк и формат каждой записи определяются реализацией.
    ///
    /// # Errors
    /// Возвращает [`ParserErr`], если хотя бы одна транзакция не может быть сериализована.
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

    fn from_text_reader(reader: Box<dyn Read>) -> Result<Vec<Self>, ParserErr> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::f32::consts::E;
    use std::io::Cursor;

    #[test]
    fn test_from_text_valid() {
        let mut fields = HashMap::new();
        fields.insert("TX_ID".to_string(), "123".to_string());
        fields.insert("TX_TYPE".to_string(), "TRANSFER".to_string());
        fields.insert("FROM_USER_ID".to_string(), "456".to_string());
        fields.insert("TO_USER_ID".to_string(), "789".to_string());
        fields.insert("AMOUNT".to_string(), "100".to_string());
        fields.insert("TIMESTAMP".to_string(), "1700000000".to_string());
        fields.insert("STATUS".to_string(), "SUCCESS".to_string());
        fields.insert("DESCRIPTION".to_string(), "\"Test transfer\"".to_string());

        let tx = TxData::from_text(&fields).unwrap();

        assert_eq!(tx.tx_id, 123);
        assert_eq!(tx.tx_type, TxType::Transfer);
        assert_eq!(tx.from_user_id, 456);
        assert_eq!(tx.to_user_id, 789);
        assert_eq!(tx.amount, 100);
        assert_eq!(tx.timestamp, 1700000000);
        assert_eq!(tx.status, Status::Success);
        assert_eq!(tx.description, "Test transfer"); 
        assert_eq!(tx.format, Format::YpBankText);
    }

    #[test]
    fn test_from_text_description_without_quotes() {
        let mut fields = HashMap::new();
        fields.insert("TX_ID".to_string(), "1".to_string());
        fields.insert("TX_TYPE".to_string(), "DEPOSIT".to_string());
        fields.insert("FROM_USER_ID".to_string(), "0".to_string());
        fields.insert("TO_USER_ID".to_string(), "100".to_string());
        fields.insert("AMOUNT".to_string(), "50".to_string());
        fields.insert("TIMESTAMP".to_string(), "1700000001".to_string());
        fields.insert("STATUS".to_string(), "SUCCESS".to_string());
        fields.insert("DESCRIPTION".to_string(), "No quotes here".to_string());

        let tx = TxData::from_text(&fields).unwrap();
        assert_eq!(tx.description, "No quotes here");
    }

    #[test]
    fn test_from_text_description_with_quotes() {
        let mut fields = HashMap::new();
        fields.insert("TX_ID".to_string(), "2".to_string());
        fields.insert("TX_TYPE".to_string(), "WITHDRAWAL".to_string());
        fields.insert("FROM_USER_ID".to_string(), "100".to_string());
        fields.insert("TO_USER_ID".to_string(), "0".to_string());
        fields.insert("AMOUNT".to_string(), "20".to_string());
        fields.insert("TIMESTAMP".to_string(), "1700000002".to_string());
        fields.insert("STATUS".to_string(), "FAILURE".to_string());
        fields.insert(
            "DESCRIPTION".to_string(),
            "\"With \"\"escaped\"\" quotes\"".to_string(),
        );

        let tx = TxData::from_text(&fields).unwrap();
        assert_eq!(tx.description, "With \"\"escaped\"\" quotes");
    }

    #[test]
    fn test_from_text_missing_field() {
        let mut fields = HashMap::new();
        fields.insert("TX_ID".to_string(), "123".to_string());
        fields.insert("TX_TYPE".to_string(), "TRANSFER".to_string());


        let err = TxData::from_text(&fields).unwrap_err();

        if let ParserErr::ParseErr { msg } = err {
            assert!(msg.to_string().contains("Missing field: FROM_USER_ID"));
        } else {
            panic!();
        }

    }

    #[test]
    fn test_from_text_invalid_tx_type() {
        let mut fields = HashMap::new();
        fields.insert("TX_ID".to_string(), "123".to_string());
        fields.insert("TX_TYPE".to_string(), "INVALID".to_string());
        fields.insert("FROM_USER_ID".to_string(), "456".to_string());
        fields.insert("TO_USER_ID".to_string(), "789".to_string());
        fields.insert("AMOUNT".to_string(), "100".to_string());
        fields.insert("TIMESTAMP".to_string(), "1700000000".to_string());
        fields.insert("STATUS".to_string(), "SUCCESS".to_string());
        fields.insert("DESCRIPTION".to_string(), "\"Test\"".to_string());

        let err = TxData::from_text(&fields).unwrap_err();

        if let ParserErr::ParseErr { msg } = err {
            assert!(msg.to_string().contains("Invalid TX_TYPE"));
        } else {
            panic!();
        }
    }

    #[test]
    fn test_from_text_invalid_status() {
        let mut fields = HashMap::new();
        fields.insert("TX_ID".to_string(), "123".to_string());
        fields.insert("TX_TYPE".to_string(), "TRANSFER".to_string());
        fields.insert("FROM_USER_ID".to_string(), "456".to_string());
        fields.insert("TO_USER_ID".to_string(), "789".to_string());
        fields.insert("AMOUNT".to_string(), "100".to_string());
        fields.insert("TIMESTAMP".to_string(), "1700000000".to_string());
        fields.insert("STATUS".to_string(), "UNKNOWN".to_string());
        fields.insert("DESCRIPTION".to_string(), "\"Test\"".to_string());

        let err = TxData::from_text(&fields).unwrap_err();

        if let ParserErr::ParseErr { msg } = err {
            assert!(msg.to_string().contains("Invalid STATUS"));
        } else {
            panic!()
        }
    }

    #[test]
    fn test_from_text_many_valid() {
        let lines = vec![
            "TX_ID: 1".to_string(),
            "TX_TYPE: DEPOSIT".to_string(),
            "FROM_USER_ID: 0".to_string(),
            "TO_USER_ID: 100".to_string(),
            "AMOUNT: 50".to_string(),
            "TIMESTAMP: 1700000001".to_string(),
            "STATUS: SUCCESS".to_string(),
            "DESCRIPTION: \"Initial deposit\"".to_string(),
            "".to_string(),
            "TX_ID: 2".to_string(),
            "TX_TYPE: TRANSFER".to_string(),
            "FROM_USER_ID: 100".to_string(),
            "TO_USER_ID: 200".to_string(),
            "AMOUNT: 25".to_string(),
            "TIMESTAMP: 1700000002".to_string(),
            "STATUS: PENDING".to_string(),
            "DESCRIPTION: \"Pending transfer\"".to_string(),
        ];

        let txs = TxData::from_text_many(&lines).unwrap();
        assert_eq!(txs.len(), 2);
        assert_eq!(txs[0].tx_id, 1);
        assert_eq!(txs[0].description, "Initial deposit");
        assert_eq!(txs[1].tx_id, 2);
        assert_eq!(txs[1].status, Status::Pending);
    }

    #[test]
    fn test_from_text_many_invalid_line() {
        let lines = vec![
            "TX_ID: 1".to_string(),
            "TX_TYPE: DEPOSIT".to_string(),
            "INVALID LINE WITHOUT COLON".to_string(), // эта строка вызовет ошибку
        ];

        let err = TxData::from_text_many(&lines).unwrap_err();

        if let ParserErr::ParseErr { msg } = err {
            assert!(msg.to_string().contains("Invalid key-value on line 3"));
        } else {
            panic!();
        }
    }

    #[test]
    fn test_from_text_reader_valid() {
        let text_content = r#"TX_ID: 3
TX_TYPE: TRANSFER
FROM_USER_ID: 200
TO_USER_ID: 300
AMOUNT: 75
TIMESTAMP: 1700000003
STATUS: PENDING
DESCRIPTION: "Pending tx"

# Comment between records

TX_ID: 4
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 400
AMOUNT: 200
TIMESTAMP: 1700000004
STATUS: SUCCESS
DESCRIPTION: "Second deposit"
"#;

        let reader = Box::new(Cursor::new(text_content.to_string()));
        let txs = TxData::from_text_reader(reader).unwrap();
        assert_eq!(txs.len(), 2);
        assert_eq!(txs[0].tx_id, 3);
        assert_eq!(txs[0].description, "Pending tx");
        assert_eq!(txs[1].tx_id, 4);
        assert_eq!(txs[1].description, "Second deposit");
    }

    #[test]
    fn test_from_text_reader_empty() {
        let reader = Box::new(Cursor::new("".to_string()));
        let txs = TxData::from_text_reader(reader).unwrap();
        assert_eq!(txs.len(), 0);
    }

    #[test]
    fn test_to_text_simple() {
        let tx = TxData {
            tx_id: 42,
            tx_type: TxType::Withdrawal,
            from_user_id: 101,
            to_user_id: 0,
            amount: 30,
            timestamp: 1700000010,
            status: Status::Success,
            description: "Cash out".to_string(),
            format: Format::YpBankText,
        };

        let text = tx.to_text().unwrap();
        let expected = r#"TX_ID: 42
TX_TYPE: WITHDRAWAL
FROM_USER_ID: 101
TO_USER_ID: 0
AMOUNT: 30
TIMESTAMP: 1700000010
STATUS: SUCCESS
DESCRIPTION: "Cash out""#;

        assert_eq!(text, expected);
    }

    #[test]
    fn test_to_text_description_with_special_chars() {
        let tx = TxData {
            tx_id: 99,
            tx_type: TxType::Transfer,
            from_user_id: 1,
            to_user_id: 2,
            amount: 10,
            timestamp: 1700000020,
            status: Status::Pending,
            description: "Amount: \"10\", note: with\nnewlines and\ttabs".to_string(),
            format: Format::YpBankText,
        };

        let text = tx.to_text().unwrap();
        assert!(text.contains(r#"DESCRIPTION: "Amount: "10", note: with"#));
        assert!(text.contains("newlines and\ttabs"));
    }

    #[test]
    fn test_to_text_many() {
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
                format: Format::YpBankText,
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
                format: Format::YpBankText,
            },
        ];

        let text = TxData::to_text_many(&txs).unwrap();
        let expected = r#"TX_ID: 1
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 10
AMOUNT: 100
TIMESTAMP: 1700000030
STATUS: SUCCESS
DESCRIPTION: "Bonus"
TX_ID: 2
TX_TYPE: TRANSFER
FROM_USER_ID: 10
TO_USER_ID: 20
AMOUNT: 25
TIMESTAMP: 1700000040
STATUS: FAILURE
DESCRIPTION: "Blocked""#;

        assert_eq!(text, expected);
    }

    #[test]
    fn test_to_text_many_empty() {
        let text = TxData::to_text_many(&[]).unwrap();
        assert_eq!(text, "");
    }

    #[test]
    fn test_roundtrip_text() {
        let original = TxData {
            tx_id: 12345,
            tx_type: TxType::Transfer,
            from_user_id: 100,
            to_user_id: 200,
            amount: 999,
            timestamp: 1700000000,
            status: Status::Success,
            description: "Roundtrip test".to_string(),
            format: Format::YpBankText,
        };

        let text = original.to_text().unwrap();
        let lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();
        let restored = TxData::from_text_many(&lines).unwrap();

        assert_eq!(restored.len(), 1);
        let restored_tx = &restored[0];

        assert_eq!(restored_tx.tx_id, original.tx_id);
        assert_eq!(restored_tx.tx_type, original.tx_type);
        assert_eq!(restored_tx.from_user_id, original.from_user_id);
        assert_eq!(restored_tx.to_user_id, original.to_user_id);
        assert_eq!(restored_tx.amount, original.amount);
        assert_eq!(restored_tx.timestamp, original.timestamp);
        assert_eq!(restored_tx.status, original.status);
        assert_eq!(restored_tx.description, original.description);
        assert_eq!(restored_tx.format, Format::YpBankText);
    }
}
