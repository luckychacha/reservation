use std::pin::Pin;

use futures::Stream;
use luckychacha_reservation::ReservationManager;
use luckychacha_reservation_abi::Reservation;
use tokio::sync::mpsc;
use tonic::Status;

mod service;

pub struct RsvpService {
    manager: ReservationManager,
}

type ReservationStream = Pin<Box<dyn Stream<Item = Result<Reservation, Status>> + Send>>;

pub struct TonicReceiverStream<T> {
    inner: mpsc::Receiver<Result<T, luckychacha_reservation_abi::Error>>,
}
