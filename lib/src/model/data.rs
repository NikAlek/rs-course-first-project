use clap::ValueEnum;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxData {
    pub tx_id: u64,
    pub tx_type: TxType,
    pub from_user_id: u64,
    pub to_user_id: u64,
    pub amount: i64,
    pub timestamp: u64,
    pub status: Status,
    pub description: String,
    pub format: Format,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxType {
    Deposit,
    Transfer,
    Withdrawal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Success,
    Failure,
    Pending,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format{
    YpBankCsv,
    YpBankText,
    YpBankBin,
}
