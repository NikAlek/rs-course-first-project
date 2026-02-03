use clap::ValueEnum;


/// Представляет одну финансовую транзакцию в системе 
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxData {
    /// Уникальный идентификатор транзакции
    pub tx_id: u64,
    /// Тип операции
    pub tx_type: TxType,
    /// Идентификатор отправителя
    pub from_user_id: u64,
    /// Идентификатор получателя 
    pub to_user_id: u64,
    /// Сумма транзакции 
    pub amount: i64,
    /// Временная метка в формате Unix timestamp 
    pub timestamp: u64,
    /// Текущий статус транзакции
    pub status: Status,
    /// Описание или комментарий к транзакции
    pub description: String,
    /// Формат, связанный с этой транзакцией
    pub format: Format,
}

/// Тип финансовой операции.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxType {
    /// Пополнение счёта (депозит)
    Deposit,
    /// Перевод между пользователями
    Transfer,
    /// Вывод средств со счёта
    Withdrawal,
}

/// Статус выполнения транзакции.
///
/// Отражает текущее состояние обработки операции в системе.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Транзакция успешно завершена
    Success,
    /// Транзакция завершилась с ошибкой
    Failure,
    /// Транзакция находится в процессе обработки
    Pending,
}

/// Поддерживаемые форматы сериализации транзакций.
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// CSV-формат с заголовком и разделителем-запятой
    ///
    /// Пример строки:
    /// ```csv
    /// TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
    /// 123,TRANSFER,1001,1002,5000,1700000000,SUCCESS,"Payment"
    /// ```
    YpBankCsv,
    /// Человекочитаемый текстовый формат
    ///
    /// Каждая транзакция представлена в виде строк с ключ=значение или фиксированной структуры.
    YpBankText,
    /// Бинарный формат для максимальной производительности
    ///
    /// Компактное представление без избыточных символов
    YpBankBin,
}