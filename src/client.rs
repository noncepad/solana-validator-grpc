use crate::{
    err::TransactionProcessingError,
    proto::{
        transaction_processing_client::TransactionProcessingClient, Empty, RentRequest,
        RentResponse, SendRequest, Status,
    },
    status_from_i32,
};
use solana_sdk::hash::Hash;

pub struct Client {
    client: TransactionProcessingClient<tonic::transport::Channel>,
}

impl Client {
    pub async fn new(server_addr: String) -> Result<Self, TransactionProcessingError> {
        let client = TransactionProcessingClient::connect(server_addr).await?;
        Ok(Self { client })
    }

    pub async fn blockhash(&mut self) -> Result<Hash, TransactionProcessingError> {
        let resp = match self.client.blockhash(Empty {}).await {
            Ok(x) => x,
            Err(e) => return Err(TransactionProcessingError::Unknown(e.to_string())),
        };
        let result = resp.get_ref();
        if result.hash.len() != 32 {
            return Err(TransactionProcessingError::PayloadWrongSize(
                result.hash.len(),
            ));
        }
        let mut h = [0u8; 32];
        h.copy_from_slice(&result.hash);

        Ok(Hash::from(h))
    }

    pub async fn rent_exemption(
        &mut self,
        account_size: usize,
    ) -> Result<u64, TransactionProcessingError> {
        let resp = match self
            .client
            .rent_exemption(RentRequest {
                size: account_size as u64,
            })
            .await
        {
            Ok(x) => x,
            Err(e) => return Err(TransactionProcessingError::Unknown(e.to_string())),
        };
        let result = resp.get_ref();

        Ok(result.lamports)
    }

    /// Send a transaction.
    pub async fn send(
        &mut self,
        serialized_tx: &[u8],
        simulate: bool,
        status: Status,
    ) -> Result<(u64, Status), TransactionProcessingError> {
        let mut transaction = Vec::with_capacity(serialized_tx.len());
        transaction.copy_from_slice(serialized_tx);
        let resp = match self
            .client
            .send(SendRequest {
                transaction,
                simulate,
                status: status as i32,
            })
            .await
        {
            Ok(x) => x,
            Err(e) => return Err(TransactionProcessingError::Unknown(e.to_string())),
        };
        let result = resp.get_ref();
        Ok((result.slot, status_from_i32(result.status)))
    }
}
