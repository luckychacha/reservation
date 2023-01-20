// 1.add dependency
// cargo add sqlx --features chrono,uuid -p luckychacha-reservation
// cargo add sqlx --features runtime-tokio-rustls,postgres -p luckychacha-reservation
// cargo add sqlx -p luckychacha-reservation

// 2.install sqlx-cli
// cargo install sqlx-cli
// sqlx migrate add init -r

// 3.init pgconfig
// touch .env
// DATABASE_URL = 'postgres://username@host:port/reservation'

// rm /usr/local/var/postgresql@11/postmaster.pid
// brew services restart postgresql

// 4.run xxx.up.sql or xxx.down.sql
// sqlx migrate run
// sqlx migrate revert

// psql -d reservation / pgcli -d reservation

// cargo nextest run
mod manager;

use async_trait::async_trait;
use luckychacha_reservation_abi::Error;
use sqlx::PgPool;

pub type ReservationId = String;
// pub type UserId = String;
// pub type ResourceId = String;

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

    async fn delete(&self, id: ReservationId) -> Result<(), Error>;

    async fn get(
        &self,
        id: ReservationId,
    ) -> Result<luckychacha_reservation_abi::Reservation, Error>;

    async fn query(
        &self,
        query: luckychacha_reservation_abi::ReservationQuery,
    ) -> Result<Vec<luckychacha_reservation_abi::Reservation>, luckychacha_reservation_abi::Error>;
}
