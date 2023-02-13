mod manager;

use async_trait::async_trait;
use luckychacha_reservation_abi::{Error, FilterPager, ReservationId};
use sqlx::PgPool;

#[derive(Debug)]
pub struct ReservationManager {
    pool: PgPool,
}

#[async_trait]
pub trait Rsvp {
    async fn reserve(
        &self,
        rsvp: luckychacha_reservation_abi::Reservation,
    ) -> Result<luckychacha_reservation_abi::Reservation, Error>;

    async fn change_status(
        &self,
        id: ReservationId,
    ) -> Result<luckychacha_reservation_abi::Reservation, Error>;

    async fn update_note(
        &self,
        id: ReservationId,
        note: String,
    ) -> Result<luckychacha_reservation_abi::Reservation, Error>;

    async fn delete(
        &self,
        id: ReservationId,
    ) -> Result<luckychacha_reservation_abi::Reservation, Error>;

    async fn get(
        &self,
        id: ReservationId,
    ) -> Result<luckychacha_reservation_abi::Reservation, Error>;

    async fn query(
        &self,
        query: luckychacha_reservation_abi::ReservationQuery,
    ) -> Result<Vec<luckychacha_reservation_abi::Reservation>, luckychacha_reservation_abi::Error>;

    async fn filter(
        &self,
        filter: luckychacha_reservation_abi::ReservationFilter,
    ) -> Result<
        (FilterPager, Vec<luckychacha_reservation_abi::Reservation>),
        luckychacha_reservation_abi::Error,
    >;
}
