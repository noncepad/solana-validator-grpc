use err::TransactionProcessingError;
use proto::Status;
use solana_sdk::hash::Hash;
use solana_sdk::signature::Signature;
use std::future::Future;
use std::pin::Pin;

pub mod proto {
    include!("proto/txproc.rs"); // Replace "mypackage" with the package name from your .proto file
    include!("proto/relay.rs"); // Replace "mypackage" with the package name from your .proto file
}
pub mod client;
pub mod err;
pub mod server;
pub fn status_from_i32(status_code: i32) -> Status {
    match status_code {
        0 => Status::Processed,
        1 => Status::Confirmed,
        2 => Status::Rooted,
        _ => Status::Processed, // Handle invalid values appropriately
    }
}

pub fn safe_divide_as_f32(numerator: u64, denominator: u64) -> f32 {
    if denominator == 0 {
        // Prevent division by zero
        1.0
    } else if 100_000_000 <= numerator || 100_000_000 <= denominator {
        1.0
    } else {
        (numerator as f32) / (denominator as f32)
    }
}

pub trait Processor: Clone + Send + Sync {
    fn rent_exemption(
        &self,
        account_size: u64,
    ) -> Pin<Box<dyn Future<Output = Result<u64, TransactionProcessingError>> + Send>>;
    //Result<u64, TransactionProcessingError>;
    fn blockhash(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Hash, TransactionProcessingError>> + Send>>;
    fn send(
        &self,
        serialized_tx: Vec<u8>,
    ) -> Pin<Box<dyn Future<Output = Result<(u64, Status), TransactionProcessingError>> + Send>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(4, 4);
    }
}
