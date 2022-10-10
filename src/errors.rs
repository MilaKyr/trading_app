use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    MessageSendError(#[from] tokio::sync::mpsc::error::SendError<String>),
    #[error(transparent)]
    LineReaderError(#[from] tokio_util::codec::LinesCodecError),
    #[error(transparent)]
    ClientError(#[from] ClientError),
}

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Unknown product. Choose between: APPLE, PEAR, TOMATO, POTATO or ONION")]
    UnknownProduct,
    #[error("Unknown action. Choose between: BUY or SELL")]
    UnknownAction,
    #[error("Invalid transaction message. Should be <Action>:<Item>")]
    InvalidTransactionMessage,
}
