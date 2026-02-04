use std::fs::File;
use std::io::{self, BufReader, BufWriter, Cursor, Read, Write, stdin, stdout};

use crate::console::commands::Resource;
use crate::model::data::{Format, TxData};
use crate::model::errors::ParserErr;
use crate::parser::concrete::bin_psrser::{TxnFromBin, TxnToBin};
use crate::parser::concrete::csv_parser::{TxnFromCsv, TxnToCsv};
use crate::parser::concrete::text_parser::{TxnFromText, TxnToText};


/// Записывает коллекцию транзакций в указанный ресурс в заданном формате.
///
/// Функция сериализует переданные транзакции в соответствии с выбранным форматом
/// и выполняет запись во внешний ресурс (например, файл или поток вывода).
/// После записи гарантируется сброс буфера через вызов `flush()`.
///
/// Возвращаемое значение
///
/// Возвращает `Ok(usize)` при успешной записи и сбросе буфера. Возвращает размер записанных данных
/// При возникновении ошибок возвращает ParserErr::ParseErr, содержащий
/// строковое описание ошибки ввода-вывода или сериализации.
pub fn write_to_resource(
    txns: &[TxData],
    resource: Resource,
    format: Format,
) -> Result<usize, ParserErr> {
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

    Ok(data_to_write.len())
}


/// Создаёт Write для указанного ресурса. Write полиморфен и зависит от resource
///
/// # Аргументы
/// * `resource` — целевой ресурс (`Console` или `File`)
///
/// # Возвращает
/// * `Ok(Box<dyn Write>)` — готовый к записи поток
/// * `Err(ParserErr)` — ошибка создания файла
///
fn write(resource: Resource) -> Result<Box<dyn Write>, ParserErr> {
    match resource {
        Resource::Console => Ok(Box::new(stdout())),
        Resource::File { path } => {
            let file =
                File::create(path).map_err(|e| ParserErr::ParseErr { msg: e.to_string() })?;
            Ok(Box::new(BufWriter::new(file)))
        },
        Resource::Memory{ data } => {
            Ok(Box::new(data))
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::model::data::{Status, TxType};
    use crate::console::commands::Resource;

    use super::*;
    use mockall::automock;
    use std::io::Cursor;
    use std::sync::{Arc, Mutex};



    fn sample_txns() -> Vec<TxData> {
        vec![
            TxData {
                tx_id: 1,
                tx_type: TxType::Deposit,
                from_user_id: 0,
                to_user_id: 100,
                amount: 1000,
                timestamp: 1700000000,
                status: Status::Success,
                description: "Initial deposit".to_string(),
                format: Format::YpBankCsv,
            },
            TxData {
                tx_id: 2,
                tx_type: TxType::Transfer,
                from_user_id: 100,
                to_user_id: 200,
                amount: 500,
                timestamp: 1700000060,
                status: Status::Pending,
                description: "Friend payment".to_string(),
                format: Format::YpBankCsv,
            },
        ]
    }

       #[test]
    fn test_ypbank_bin_format() {
        let txns = sample_txns();

        let size = write_to_resource(&txns, Resource::Memory{data: Cursor::new(vec![])}, Format::YpBankBin)
            .expect("binary write should succeed");

        assert_eq!(size, 137)
    }

        #[test]
    fn test_ypbank_csv_format() {
        let txns = sample_txns();

        let size = write_to_resource(&txns, Resource::Memory{data: Cursor::new(vec![])}, Format::YpBankCsv)
            .expect("binary write should succeed");

        assert_eq!(size, 116)
    }

    #[test]
    fn test_ypbank_text_format() {
        let txns = sample_txns();

        let size = write_to_resource(&txns, Resource::Memory{data: Cursor::new(vec![])}, Format::YpBankText)
            .expect("binary write should succeed");

        assert_eq!(size, 280)
    }
    
}