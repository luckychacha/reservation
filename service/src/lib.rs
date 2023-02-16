mod service;
#[cfg(test)]
mod test_utils;

use anyhow::Result;
use futures::Stream;
use luckychacha_reservation::ReservationManager;
use luckychacha_reservation_abi::{
    reservation_service_server::ReservationServiceServer, Config, Reservation,
};
use std::pin::Pin;
use tokio::sync::mpsc;
use tonic::{transport::Server, Status};

pub struct RsvpService {
    manager: ReservationManager,
}

type ReservationStream = Pin<Box<dyn Stream<Item = Result<Reservation, Status>> + Send>>;

pub struct TonicReceiverStream<T> {
    inner: mpsc::Receiver<Result<T, luckychacha_reservation_abi::Error>>,
}

pub async fn start_server(config: &Config) -> Result<(), anyhow::Error> {
    let addr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    let svc = RsvpService::from_config(config).await?;
    let svc = ReservationServiceServer::new(svc);

    println!("Listening on: {addr}");

    Server::builder().add_service(svc).serve(addr).await?;
    Ok(())
}
