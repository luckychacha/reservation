use std::pin::Pin;

use futures::Stream;
use luckychacha_reservation::ReservationManager;
use luckychacha_reservation_abi::Reservation;
use tonic::Status;

mod service;

pub struct RsvpService {
    manager: ReservationManager,
}

type ReservationStream = Pin<Box<dyn Stream<Item = Result<Reservation, Status>> + Send>>;
