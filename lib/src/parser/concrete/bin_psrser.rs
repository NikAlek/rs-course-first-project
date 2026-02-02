use csv::{ReaderBuilder, StringRecord};
use std::io::Write;
use std::io::{Cursor, Read};

use crate::model::data::Format;
use crate::model::data::Status;
use crate::model::data::TxData;
use crate::model::data::TxType;
use crate::model::errors::ParserErr;

const BIN_MAGIC: [u8; 4] = *b"YPBN";

/// Трейт для парсинга транзакций из бинарного представления.
///
/// Предоставляет два способа десериализации:
/// - `from_bin`: парсит одну транзакцию из среза байтов (`&[u8]`).
/// - `from_bin_reader`: читает и парсит **несколько** транзакций из объекта, реализующего `Read`,
///   что полезно при работе с файлами или сетевыми потоками.
pub trait TxnFromBin {
    /// Парсит одну транзакцию из бинарного среза.
    ///
    /// # Errors
    /// Возвращает `ParserErr`, если входные данные повреждены, неполны или не соответствуют ожидаемому формату.
    fn from_bin(body: &[u8]) -> Result<TxData, ParserErr>;

    /// Парсит последовательность транзакций из потока байтов.
    ///
    /// Принимает `Box<dyn Read>`, чтобы поддерживать произвольные источники данных (файлы, сокеты и т.д.).
    /// Предполагается, что поток содержит сериализованные транзакции в известном формате,
    /// например, с префиксом длины или разделителями.
    ///
    /// # Errors
    /// Возвращает `ParserErr`, если чтение или парсинг любой из транзакций завершилось неудачей.
    fn from_bin_reader(reader: Box<dyn Read>) -> Result<Vec<TxData>, ParserErr>;
}

/// Трейт для сериализации транзакций в бинарное представление.
///
/// Поддерживает как сериализацию одной транзакции, так и пакетную сериализацию.
pub trait TxnToBin {
    /// Сериализует одну транзакцию в бинарный формат.
    ///
    /// # Errors
    /// Возвращает `ParserErr`, если сериализация невозможна (например, из-за внутренней ошибки логики).
    fn to_bin(&self) -> Result<Vec<u8>, ParserErr>;

    /// Сериализует множество транзакций в единый бинарный блок.
    ///
    /// Конкретный формат (например, конкатенация, длина + данные и т.п.) определяется реализацией.
    ///
    /// # Errors
    /// Возвращает `ParserErr`, если сериализация хотя бы одной транзакции завершилась неудачей.
    fn to_bin_many(many: &[Self]) -> Result<Vec<u8>, ParserErr>
    where
        Self: Sized;
}

impl TxnFromBin for TxData {
    fn from_bin(body: &[u8]) -> Result<Self, ParserErr> {
        use byteorder::{BigEndian, ReadBytesExt};
        let mut cursor = std::io::Cursor::new(body);

        let tx_id = cursor
            .read_u64::<BigEndian>()
            .map_err(|e| ParserErr::ParseErr { msg: e.to_string() })?;
        let tx_type = match cursor
            .read_u8()
            .map_err(|e| ParserErr::ParseErr { msg: e.to_string() })?
        {
            0 => TxType::Deposit,
            1 => TxType::Transfer,
            2 => TxType::Withdrawal,
            v => {
                return Err(ParserErr::ParseErr {
                    msg: format!("Invalid TX_TYPE: {}", v),
                });
            }
        };
        let from_user_id = cursor
            .read_u64::<BigEndian>()
            .map_err(|e| ParserErr::ParseErr { msg: e.to_string() })?;
        let to_user_id = cursor
            .read_u64::<BigEndian>()
            .map_err(|e| ParserErr::ParseErr { msg: e.to_string() })?;
        let amount = cursor
            .read_i64::<BigEndian>()
            .map_err(|e| ParserErr::ParseErr { msg: e.to_string() })?;
        let timestamp = cursor
            .read_u64::<BigEndian>()
            .map_err(|e| ParserErr::ParseErr { msg: e.to_string() })?;
        let status = match cursor
            .read_u8()
            .map_err(|e| ParserErr::ParseErr { msg: e.to_string() })?
        {
            0 => Status::Success,
            1 => Status::Failure,
            2 => Status::Pending,
            v => {
                return Err(ParserErr::ParseErr {
                    msg: format!("Invalid STATUS: {}", v),
                });
            }
        };
        let desc_len = cursor
            .read_u32::<BigEndian>()
            .map_err(|e| ParserErr::ParseErr { msg: e.to_string() })?
            as usize;
        if cursor.position() as usize + desc_len > body.len() {
            return Err(ParserErr::ParseErr {
                msg: "DESCRIPTION length exceeds body".into(),
            });
        }
        let mut desc_bytes = vec![0u8; desc_len];
        cursor
            .read_exact(&mut desc_bytes)
            .map_err(|e| ParserErr::ParseErr { msg: e.to_string() })?;
        let description = String::from_utf8(desc_bytes).map_err(|e| ParserErr::ParseErr {
            msg: format!("Invalid UTF-8 in DESCRIPTION: {}", e),
        })?;

        Ok(TxData {
            tx_id: tx_id,
            tx_type: tx_type,
            from_user_id: from_user_id,
            to_user_id: to_user_id,
            amount: amount,
            timestamp: timestamp,
            status: status,
            description: description,
            format: Format::YpBankBin,
        })
    }

    fn from_bin_reader(mut reader: Box<dyn Read>) -> Result<Vec<Self>, ParserErr> {
        let mut transactions = Vec::new();
        let mut buf = Vec::new();

        loop {
            let mut magic = [0u8; 4];
            if reader.read_exact(&mut magic).is_err() {
                break;
            }
            if magic != BIN_MAGIC {
                return Err(ParserErr::ParseErr {
                    msg: "Invalid MAGIC number".into(),
                });
            }

            let record_size = {
                let mut size_bytes = [0u8; 4];
                reader
                    .read_exact(&mut size_bytes)
                    .map_err(|e| ParserErr::ParseErr { msg: e.to_string() })?;
                u32::from_be_bytes(size_bytes) as usize
            };

            buf.resize(record_size, 0);
            reader
                .read_exact(&mut buf)
                .map_err(|e| ParserErr::ParseErr { msg: e.to_string() })?;

            let tx = Self::from_bin(&buf)?;
            transactions.push(tx);
        }

        Ok(transactions)
    }
}

impl TxnToBin for TxData {
    fn to_bin(&self) -> Result<Vec<u8>, ParserErr> {
        use byteorder::{BigEndian, WriteBytesExt};
        let mut body = Vec::new();

        body.write_u64::<BigEndian>(self.tx_id)
            .map_err(|e| ParserErr::SerializeErr { msg: e.to_string() })?;
        body.write_u8(match self.tx_type {
            TxType::Deposit => 0,
            TxType::Transfer => 1,
            TxType::Withdrawal => 2,
        })
        .map_err(|e| ParserErr::SerializeErr { msg: e.to_string() })?;
        body.write_u64::<BigEndian>(self.from_user_id)
            .map_err(|e| ParserErr::SerializeErr { msg: e.to_string() })?;
        body.write_u64::<BigEndian>(self.to_user_id)
            .map_err(|e| ParserErr::SerializeErr { msg: e.to_string() })?;
        body.write_i64::<BigEndian>(self.amount)
            .map_err(|e| ParserErr::SerializeErr { msg: e.to_string() })?;
        body.write_u64::<BigEndian>(self.timestamp)
            .map_err(|e| ParserErr::SerializeErr { msg: e.to_string() })?;
        body.write_u8(match self.status {
            Status::Success => 0,
            Status::Failure => 1,
            Status::Pending => 2,
        })
        .map_err(|e| ParserErr::SerializeErr { msg: e.to_string() })?;

        let desc_bytes = self.description.as_bytes();
        body.write_u32::<BigEndian>(desc_bytes.len() as u32)
            .map_err(|e| ParserErr::SerializeErr { msg: e.to_string() })?;
        body.write_all(desc_bytes)
            .map_err(|e| ParserErr::SerializeErr { msg: e.to_string() })?;

        // Теперь формируем полную запись: MAGIC + RECORD_SIZE + body
        let mut full = Vec::with_capacity(8 + body.len());
        full.extend_from_slice(b"YPBN");
        full.write_u32::<BigEndian>(body.len() as u32)
            .map_err(|e| ParserErr::SerializeErr { msg: e.to_string() })?;
        full.extend_from_slice(&body);

        Ok(full)
    }

    fn to_bin_many(transactions: &[Self]) -> Result<Vec<u8>, ParserErr> {
        let mut all = Vec::new();
        for tx in transactions {
            all.extend_from_slice(&tx.to_bin()?);
        }
        Ok(all)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{f32::consts::E, io::Cursor};

    #[test]
    fn test_to_bin_and_from_bin_roundtrip() {
        let original = TxData {
            tx_id: 12345,
            tx_type: TxType::Transfer,
            from_user_id: 100,
            to_user_id: 200,
            amount: 999_000_000_000i64, 
            timestamp: 1700000000,
            status: Status::Success,
            description: "Test binary transaction".to_string(),
            format: Format::YpBankBin,
        };

        let bin_data = original.to_bin().unwrap();
        let restored = TxData::from_bin_reader(Box::new(Cursor::new(bin_data))).unwrap();

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
        assert_eq!(restored_tx.format, Format::YpBankBin);
    }

    #[test]
    fn test_from_bin_description_length_exceeds_body() {
        let body: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0, 1, // tx_id
            0, // tx_type
            0, 0, 0, 0, 0, 0, 0, 0, // from_user_id
            0, 0, 0, 0, 0, 0, 0, 0, // to_user_id
            0, 0, 0, 0, 0, 0, 0, 0, // amount
            0, 0, 0, 0, 0, 0, 0, 0, // timestamp
            0, // status
            0, 0, 0, 10, 
            1, 2, 3, 4, 5,
        ];

        let err = TxData::from_bin(&body).unwrap_err();

        if let ParserErr::ParseErr { msg } = err {
            assert!(msg.to_string().contains("DESCRIPTION length exceeds body"));
        } else {
            panic!()
        }
    }

    #[test]
    fn test_from_bin_reader_valid_multiple() {
        let tx1 = TxData {
            tx_id: 1,
            tx_type: TxType::Deposit,
            from_user_id: 0,
            to_user_id: 100,
            amount: 1000,
            timestamp: 1700000000,
            status: Status::Success,
            description: "First deposit".to_string(),
            format: Format::YpBankBin,
        };
        let tx2 = TxData {
            tx_id: 2,
            tx_type: TxType::Withdrawal,
            from_user_id: 100,
            to_user_id: 0,
            amount: 500,
            timestamp: 1700000001,
            status: Status::Failure,
            description: "Failed withdrawal".to_string(),
            format: Format::YpBankBin,
        };

        let bin_data = TxData::to_bin_many(&[tx1, tx2]).unwrap();
        let reader = Box::new(Cursor::new(bin_data));
        let transactions = TxData::from_bin_reader(reader).unwrap();

        assert_eq!(transactions.len(), 2);
        assert_eq!(transactions[0].tx_id, 1);
        assert_eq!(transactions[1].tx_id, 2);
        assert_eq!(transactions[0].description, "First deposit");
        assert_eq!(transactions[1].description, "Failed withdrawal");
    }

    #[test]
    fn test_from_bin_reader_empty() {
        let reader = Box::new(Cursor::new(Vec::<u8>::new()));
        let transactions = TxData::from_bin_reader(reader).unwrap();
        assert_eq!(transactions.len(), 0);
    }

    #[test]
    fn test_from_bin_reader_invalid_magic() {
        let mut data = Vec::new();
        data.extend_from_slice(b"INVALID"); // wrong magic
        let reader = Box::new(Cursor::new(data));
        let err = TxData::from_bin_reader(reader).unwrap_err();
        if let ParserErr::ParseErr { msg } = err {
            assert!(msg.to_string().contains("Invalid MAGIC number"));
        } else {
            panic!()
        }
    }

    #[test]
    fn test_to_bin_description_with_special_chars() {
        let tx = TxData {
            tx_id: 999,
            tx_type: TxType::Transfer,
            from_user_id: 123,
            to_user_id: 456,
            amount: -123456789i64,
            timestamp: 9999999999,
            status: Status::Pending,
            description: "Special chars: 🚀\n\t\"\\'".to_string(),
            format: Format::YpBankBin,
        };

        let bin_data = tx.to_bin().unwrap();
        let restored = TxData::from_bin_reader(Box::new(Cursor::new(bin_data))).unwrap();
        assert_eq!(restored.len(), 1);
        assert_eq!(restored[0].description, tx.description);
    }

    #[test]
    fn test_to_bin_many_empty() {
        let bin_data = TxData::to_bin_many(&[]).unwrap();
        assert_eq!(bin_data.len(), 0);

        let reader = Box::new(Cursor::new(bin_data));
        let transactions = TxData::from_bin_reader(reader).unwrap();
        assert_eq!(transactions.len(), 0);
    }

    #[test]
    fn test_to_bin_structure() {
        let tx = TxData {
            tx_id: 1,
            tx_type: TxType::Deposit,
            from_user_id: 0,
            to_user_id: 1,
            amount: 100,
            timestamp: 1,
            status: Status::Success,
            description: "test".to_string(),
            format: Format::YpBankBin,
        };

        let full_record = tx.to_bin().unwrap();


        assert_eq!(&full_record[0..4], b"YPBN");


        let record_size = u32::from_be_bytes([
            full_record[4],
            full_record[5],
            full_record[6],
            full_record[7],
        ]);
        let expected_body_len = 8 + 1 + 8 + 8 + 8 + 8 + 1 + 4 + 4; 
        assert_eq!(record_size as usize, expected_body_len);

   
        let body = &full_record[8..];
        assert_eq!(body.len(), expected_body_len);
    }
}
