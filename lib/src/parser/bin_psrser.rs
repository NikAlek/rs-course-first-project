use csv::{ReaderBuilder, StringRecord};
use std::io::Write;
use std::io::{Cursor, Read};

use crate::model::data::Format;
use crate::model::data::Status;
use crate::model::data::TxData;
use crate::model::data::TxType;
use crate::model::errors::ParserErr;

const BIN_MAGIC: [u8; 4] = *b"YPBN";

pub trait TxnFromBin {
    fn from_bin(body: &[u8]) -> Result<TxData, ParserErr>;
    fn from_bin_reader(reader: Box<dyn Read>) -> Result<Vec<TxData>, ParserErr>;
}

pub trait TxnToBin   {
    fn to_bin(&self) -> Result<Vec<u8>, ParserErr>;
    fn to_bin_many(many: &[Self]) -> Result<Vec<u8>, ParserErr> where Self: Sized;
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

        body.write_u64::<BigEndian>(self.tx_id).map_err(|e| ParserErr::SerializeErr{ msg : e.to_string()})?;
        body.write_u8(match self.tx_type {
            TxType::Deposit => 0,
            TxType::Transfer => 1,
            TxType::Withdrawal => 2,
        }).map_err(|e| ParserErr::SerializeErr{ msg : e.to_string()})?;
        body.write_u64::<BigEndian>(self.from_user_id).map_err(|e| ParserErr::SerializeErr{ msg : e.to_string()})?;
        body.write_u64::<BigEndian>(self.to_user_id).map_err(|e| ParserErr::SerializeErr{ msg : e.to_string()})?;
        body.write_i64::<BigEndian>(self.amount).map_err(|e| ParserErr::SerializeErr{ msg : e.to_string()})?;
        body.write_u64::<BigEndian>(self.timestamp).map_err(|e| ParserErr::SerializeErr{ msg : e.to_string()})?;
        body.write_u8(match self.status {
            Status::Success => 0,
            Status::Failure => 1,
            Status::Pending => 2,
        }).map_err(|e| ParserErr::SerializeErr{ msg : e.to_string()})?;

        let desc_bytes = self.description.as_bytes();
        body.write_u32::<BigEndian>(desc_bytes.len() as u32).map_err(|e| ParserErr::SerializeErr{ msg : e.to_string()})?;
        body.write_all(desc_bytes).map_err(|e| ParserErr::SerializeErr{ msg : e.to_string()})?;

        // Теперь формируем полную запись: MAGIC + RECORD_SIZE + body
        let mut full = Vec::with_capacity(8 + body.len());
        full.extend_from_slice(b"YPBN");
        full.write_u32::<BigEndian>(body.len() as u32).map_err(|e| ParserErr::SerializeErr{ msg : e.to_string()})?;
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
