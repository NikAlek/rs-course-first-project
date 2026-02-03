use std::fs::File;

use std::io::{self, BufReader, BufWriter, Read, Write, stdin, stdout};

use crate::console::commands::Resource;
use crate::model::data::{Format, TxData};
use crate::model::errors::ParserErr;
use crate::parser::concrete::bin_psrser::TxnFromBin;
use crate::parser::concrete::csv_parser::TxnFromCsv;
use crate::parser::concrete::text_parser::TxnFromText;


/// Читает транзакции из указанного ресурса в заданном формате.
///
/// # Аргументы
/// * `resource` — источник данных (`Console` или `File`)
/// * `format` — формат данных (`YpBankBin`, `YpBankCsv`, `YpBankText`)
///
/// # Возвращает
/// * `Ok(Vec<TxData>)` — вектор распарсенных транзакций
/// * `Err(ParserErr)` — ошибка чтения файла или парсинга данных
///
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



#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_csv_format_returns_data() {
        // Мокаем источник данных через Cursor (реализует Read)
        let mock_csv = Cursor::new(
            "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n42,WITHDRAWAL,101,0,30,1700000010,SUCCESS,\"Cash out\""
        );
        let result = read_from_resource(Box::new(mock_csv), &Format::YpBankCsv);
        assert!(result.is_ok());
        let transactions = result.unwrap();
        assert!(!transactions.is_empty());
    }


    #[test]
    fn test_read_file_not_found_returns_error() {
        // Пытаемся прочитать несуществующий файл
        let resource = Resource::File {
            path: "/this/path/does/not/exist.tx".to_string().into(),
        };
        let result = read(&resource, &Format::YpBankCsv);
        
        assert!(result.is_err());
    }
}