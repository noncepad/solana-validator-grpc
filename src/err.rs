use std::io;

use tonic::Code;

#[derive(Debug)]
pub enum TransactionProcessingError {
    NetworkError,       // Variant for network-related errors
    InsufficientBuffer, // the buffer requested is bigger than what is allowed
    OutOfRange,         // the index is not in the array
    Unknown(String),    // just use a string
    GenericError(Box<dyn std::error::Error>),
    PayloadWrongSize(usize),
}

impl std::fmt::Display for TransactionProcessingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionProcessingError::NetworkError => write!(f, "Network Error occurred"),
            TransactionProcessingError::InsufficientBuffer => {
                write!(f, "Requested buffer is tool large")
            }
            TransactionProcessingError::OutOfRange => write!(f, "index is out of range"),
            TransactionProcessingError::Unknown(e) => write!(f, "unknown: {}", e),
            TransactionProcessingError::GenericError(e) => write!(f, "generic: {}", e),
            TransactionProcessingError::PayloadWrongSize(s) => {
                write!(f, "payload {} is too big", s)
            }
        }
    }
}
impl From<TransactionProcessingError> for io::Error {
    fn from(err: TransactionProcessingError) -> io::Error {
        match err {
            _ => io::Error::new(io::ErrorKind::Other, err.to_string()),
        }
    }
}
impl std::error::Error for TransactionProcessingError {}
impl From<io::Error> for TransactionProcessingError {
    fn from(value: io::Error) -> Self {
        TransactionProcessingError::GenericError(Box::new(value))
    }
}
impl From<tonic::transport::Error> for TransactionProcessingError {
    fn from(value: tonic::transport::Error) -> Self {
        TransactionProcessingError::GenericError(Box::new(value))
    }
}
impl From<TransactionProcessingError> for tonic::Status {
    fn from(value: TransactionProcessingError) -> Self {
        match value {
            TransactionProcessingError::NetworkError => {
                tonic::Status::new(Code::Internal, value.to_string())
            }
            TransactionProcessingError::InsufficientBuffer => {
                tonic::Status::new(Code::Internal, value.to_string())
            }
            TransactionProcessingError::OutOfRange => {
                tonic::Status::new(Code::Internal, value.to_string())
            }
            TransactionProcessingError::Unknown(e) => tonic::Status::new(Code::Internal, e),
            TransactionProcessingError::GenericError(e) => {
                tonic::Status::new(Code::Internal, e.to_string())
            }
            TransactionProcessingError::PayloadWrongSize(s) => {
                tonic::Status::new(Code::Internal, format!("payload too big {}", s))
            }
        }
    }
}
