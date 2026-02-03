use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum CommonErr {
    /// Ошибка ввода-вывода (чтение/запись файлов, консоли и т.д.)
    #[error("io error")]
    IO(#[from] IoErr),

    /// Ошибка парсинга или сериализации данных
    #[error("parser error")]
    Parser(#[from] ParserErr),

    /// Неожиданная ошибка, не подпадающая под другие категории
    #[error("unexpected error")]
    Unexpected,
}

/// Ошибки, связанные с операциями ввода-вывода.
#[derive(Error, Debug, Clone)]
pub enum IoErr {
    /// Ошибка при чтении входных данных
    #[error("io -> input error")]
    InputErr,

    /// Ошибка при записи выходных данных
    #[error("io -> output error")]
    OutputErr,
}

/// Ошибки, связанные с парсингом и сериализацией данных.
#[derive(Error, Debug, Clone)]
pub enum ParserErr {
    /// Ошибка при десериализации (парсинге) входных данных
    #[error("parser -> global error")]
    ParseErr { msg: String },

    /// Ошибка при сериализации данных для вывода
    #[error("serealize -> global error")]  // Примечание: опечатка в "serialize"
    SerializeErr { msg: String },
}