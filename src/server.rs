use crate::{
    err::TransactionProcessingError,
    proto::{
        self,
        capacity_server::{Capacity, CapacityServer},
        transaction_processing_server::{TransactionProcessing, TransactionProcessingServer},
        BlockhashResponse, CapacityRequest, CapacityStatus, Empty, RentRequest, RentResponse,
        SendRequest, Status, TransactionResult,
    },
    safe_divide_as_f32, Processor,
};

use log::warn;
use std::{
    error::Error,
    pin::Pin,
    sync::{atomic::AtomicU64, Arc},
    time::Duration,
};
use tokio::{
    sync::{mpsc, Notify, RwLock},
    time::sleep,
};
use tokio_stream::{self, wrappers::ReceiverStream, Stream, StreamExt};
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
    let capacity = CapacityImpl::default();
    let service = ProcessorImpl::new(hook, capacity.clone());
    match Server::builder()
        .add_service(TransactionProcessingServer::new(service))
        .add_service(CapacityServer::new(capacity))
        .serve(addr)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(TransactionProcessingError::GenericError(Box::new(e))),
    }
}

#[derive(Debug, Clone, Default)]
pub struct CapacityImpl {
    used: Arc<AtomicU64>,  // increment this when sending transactions
    total: Arc<AtomicU64>, // put swqos tps here in bytes per second
}
impl CapacityImpl {
    pub fn total_adjust(&self, total: u64) {
        self.total.store(total, std::sync::atomic::Ordering::SeqCst)
    }
    pub fn mark_sending(&self, sent: u64) -> u64 {
        self.used
            .fetch_add(sent, std::sync::atomic::Ordering::SeqCst)
    }
    pub fn mark_sent(&self, sent: u64) -> u64 {
        self.used
            .fetch_sub(sent, std::sync::atomic::Ordering::SeqCst)
    }
}

#[tonic::async_trait]
impl Capacity for CapacityImpl {
    #[doc = " Server streaming response type for the OnStatus method."]
    type OnStatusStream = Pin<Box<dyn Stream<Item = Result<CapacityStatus, tonic::Status>> + Send>>;

    /// Stream capacity updates to the reverse proxy and potential users.
    async fn on_status(
        &self,
        _request: tonic::Request<CapacityRequest>,
    ) -> std::result::Result<tonic::Response<Self::OnStatusStream>, tonic::Status> {
        let (tx, rx) = mpsc::channel(128);
        let used = self.used.clone();
        let total = self.total.clone();
        tokio::spawn(async move {
            loop {
                // loop every 30 seconds and set the utilization raio
                let u = used.load(std::sync::atomic::Ordering::SeqCst);
                let t = total.load(std::sync::atomic::Ordering::SeqCst);
                let ratio = safe_divide_as_f32(u, t);
                // send capacity as full
                match tx
                    .send(Ok(CapacityStatus {
                        utilization_ratio: ratio,
                    }))
                    .await
                {
                    Ok(_) => {}
                    Err(e) => {
                        warn!("failed to send: {}", e);
                        break;
                    }
                };
                sleep(Duration::from_secs(30)).await;
            }
        });
        let output_stream = ReceiverStream::new(rx);
        Ok(tonic::Response::new(
            Box::pin(output_stream) as Self::OnStatusStream
        ))
    }
}

#[derive(Debug, Clone)]
pub struct ProcessorImpl<T: Processor> {
    hook: T,
    capacity: CapacityImpl,
}

impl<T: Processor> ProcessorImpl<T> {
    pub fn new(hook: T, capacity: CapacityImpl) -> Self {
        Self { hook, capacity }
    }
}

#[tonic::async_trait]
impl<T: Processor + 'static> TransactionProcessing for ProcessorImpl<T> {
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
        let len = data.len() as u64;
        self.capacity.mark_sending(len);
        let (slot, status) = self.hook.send(data).await?;
        self.capacity.mark_sent(len);
        Ok(tonic::Response::new(TransactionResult {
            slot,
            status: status as i32,
        }))
    }
}

pub fn safe_divide_checked(numerator: u64, denominator: u64) -> Option<f64> {
    if let Some(int_result) = numerator.checked_div(denominator) {
        Some(int_result as f64)
    } else {
        None
    }
}
