use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write, stdin, stdout};

use crate::console::commands::Resource;
use crate::model::data::{Format, TxData};
use crate::model::errors::ParserErr;
use crate::parser::concrete::bin_psrser::{TxnFromBin, TxnToBin};
use crate::parser::concrete::csv_parser::{TxnFromCsv, TxnToCsv};
use crate::parser::concrete::text_parser::{TxnFromText, TxnToText};

/// Создаёт writer для указанного ресурса.
///
/// # Аргументы
/// * `resource` — целевой ресурс (`Console` или `File`)
///
/// # Возвращает
/// * `Ok(Box<dyn Write>)` — готовый к записи поток
/// * `Err(ParserErr)` — ошибка создания файла
///
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


#[cfg(test)]
mod tests {
    use crate::model::data::{Status, TxType};

    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_write_csv_format() {
        // Arrange: мокаем вывод через Cursor
        let txns = vec![
           TxData {
            tx_id: 42,
            tx_type: TxType::Withdrawal,
            from_user_id: 101,
            to_user_id: 0,
            amount: 30,
            timestamp: 1700000010,
            status: Status::Success,
            description: "Cash out".to_string(),
            format: Format::YpBankCsv,
        },
           TxData {
            tx_id: 43,
            tx_type: TxType::Withdrawal,
            from_user_id: 101,
            to_user_id: 0,
            amount: 30,
            timestamp: 1700000010,
            status: Status::Success,
            description: "Cash out".to_string(),
            format: Format::YpBankCsv,
        },
        ];
        let mut buffer = Vec::new();
        let mut mock_writer = Cursor::new(&mut buffer);

        // Act: сериализуем вручную (логика из write_to_resource)
        let lines: Vec<String> = txns.iter().map(|t| t.to_csv().unwrap()).collect();
        let content = lines.join("\n");
        mock_writer.write_all(content.as_bytes()).unwrap();
        mock_writer.flush().unwrap();

        // Assert
        let result = String::from_utf8(buffer).unwrap();
        assert!(result.contains("42"));
        assert!(result.contains("43"));
    }

    #[test]
    fn test_write_bin_format() {
        // Arrange
        let txns = vec![
       TxData {
            tx_id: 42,
            tx_type: TxType::Withdrawal,
            from_user_id: 101,
            to_user_id: 0,
            amount: 30,
            timestamp: 1700000010,
            status: Status::Success,
            description: "Cash out".to_string(),
            format: Format::YpBankCsv,
        }
        ];
        let mut buffer = Vec::new();
        let mut mock_writer = Cursor::new(&mut buffer);

        // Act: сериализуем бинарные данные
        let mut bin_data = Vec::new();
        for txn in &txns {
            bin_data.extend(txn.to_bin().unwrap());
        }
        mock_writer.write_all(&bin_data).unwrap();
        mock_writer.flush().unwrap();

        // Assert
        assert!(!buffer.is_empty());

        assert!(buffer.len() > 0);
    }

    #[test]
    fn test_write_text_format() {
        // Arrange
        let txns = vec![
           TxData {
            tx_id: 42,
            tx_type: TxType::Withdrawal,
            from_user_id: 101,
            to_user_id: 0,
            amount: 30,
            timestamp: 1700000010,
            status: Status::Success,
            description: "Cash out".to_string(),
            format: Format::YpBankCsv,
        },
            TxData {
            tx_id: 43,
            tx_type: TxType::Withdrawal,
            from_user_id: 101,
            to_user_id: 0,
            amount: 30,
            timestamp: 1700000010,
            status: Status::Success,
            description: "Cash out".to_string(),
            format: Format::YpBankCsv,
        },
        ];
        let mut buffer = Vec::new();
        let mut mock_writer = Cursor::new(&mut buffer);

        // Act: сериализуем в текстовый формат
        let lines: Vec<String> = txns.iter().map(|t| t.to_text().unwrap()).collect();
        let content = lines.join("\n");
        mock_writer.write_all(content.as_bytes()).unwrap();
        mock_writer.flush().unwrap();

        // Assert
        let result = String::from_utf8(buffer).unwrap();
        assert!(result.contains("42"));
        assert!(result.contains("43"));
    }
}