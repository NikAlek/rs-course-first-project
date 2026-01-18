use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum CommonErr {
    #[error("io error")]
    IO(#[from] IoErr),

    #[error("parser error")]
    Parser(#[from] ParserErr),

    #[error("unexpected error")]
    Unexpected,
}


#[derive(Error, Debug, Clone)]
pub enum IoErr {
    #[error("io -> input error")]
    InputErr,
    #[error("io -> output error")]
    OutputErr,
}

#[derive(Error, Debug, Clone)]
pub enum ParserErr {
    #[error("parser -> global error")]
    ParseErr{ msg: String },
    #[error("serealize -> global error")]
    SerializeErr{ msg: String },
}