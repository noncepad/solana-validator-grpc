use crate::{
    err::TransactionProcessingError,
    proto::{
        transaction_processing_server::{TransactionProcessing, TransactionProcessingServer},
        BlockhashResponse, Empty, RentRequest, RentResponse, SendRequest, TransactionResult,
    },
    Processor,
};
use std::{
    error::Error,
    sync::{atomic::AtomicU64, Arc},
};
use tokio::sync::{mpsc, Notify, RwLock};
use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt};
use tonic::transport::Server;

/// Run a grpc server.
pub async fn run<T: Processor + 'static>(
    listen_address: String,
    hook: T,
) -> Result<(), TransactionProcessingError> {
    let addr = match listen_address.parse() {
        Ok(x) => x,
        Err(_e) => return Err(TransactionProcessingError::NetworkError),
    };
    let service = ProcessorServer::new(hook);
    match Server::builder()
        .add_service(TransactionProcessingServer::new(service))
        .serve(addr)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(TransactionProcessingError::GenericError(Box::new(e))),
    }
}

#[derive(Debug, Clone)]
pub struct ProcessorServer<T: Processor> {
    hook: T,
}
impl<T: Processor> ProcessorServer<T> {
    pub fn new(hook: T) -> Self {
        Self { hook }
    }
}

#[tonic::async_trait]
impl<T: Processor + 'static> TransactionProcessing for ProcessorServer<T> {
    #[doc = "get the current blockhash"]
    async fn blockhash(
        &self,
        _request: tonic::Request<Empty>,
    ) -> std::result::Result<tonic::Response<BlockhashResponse>, tonic::Status> {
        let bh = self.hook.blockhash().await?;
        Ok(tonic::Response::new(BlockhashResponse {
            hash: bh.to_bytes().to_vec(),
        }))
    }

    #[doc = "Get the rent exemption for a block"]
    async fn rent_exemption(
        &self,
        request: tonic::Request<RentRequest>,
    ) -> std::result::Result<tonic::Response<RentResponse>, tonic::Status> {
        let r = request.get_ref();
        Ok(tonic::Response::new(RentResponse {
            lamports: self.hook.rent_exemption(r.size).await?,
        }))
    }

    #[doc = "send a transaction"]
    async fn send(
        &self,
        request: tonic::Request<SendRequest>,
    ) -> std::result::Result<tonic::Response<TransactionResult>, tonic::Status> {
        let r = request.get_ref();
        let mut data = Vec::with_capacity(r.transaction.len());
        data.copy_from_slice(&r.transaction);
        let (slot, status) = self.hook.send(data).await?;
        Ok(tonic::Response::new(TransactionResult {
            slot,
            status: status as i32,
        }))
    }
}
