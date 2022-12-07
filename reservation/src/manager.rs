use crate::{ReservationError, ReservationId, ReservationManager, Rsvp};
use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use luckychacha_reservation_abi::{convert_to_utc_time, Reservation, ReservationQuery};
use sqlx::postgres::types::PgRange;
use sqlx::Row;
use std::time::SystemTime;

#[async_trait]
impl Rsvp for ReservationManager {
    async fn reserve(&self, mut rsvp: Reservation) -> Result<Reservation, ReservationError> {
        if rsvp.start.is_none() || rsvp.end.is_none() {
            return Err(ReservationError::InvalidTime);
        }

        let status = luckychacha_reservation_abi::ReservationStatus::from_i32(rsvp.status)
            .unwrap_or(luckychacha_reservation_abi::ReservationStatus::Pending);
        let start = convert_to_utc_time(rsvp.start.clone().unwrap());
        let end = convert_to_utc_time(rsvp.end.clone().unwrap());

        let timespan: PgRange<DateTime<Utc>> = (start..end).into();

        // let range: PgRange<> = (rsvp.start..rsvp.end).into();
        // generate a insert sql for reservation.

        // rsvp.user_id,
        //             rsvp.resource_id,
        //             timespan,
        //             rsvp.note,
        //             rsvp.status
        let id = sqlx::query(
            "INSERT INTO reservation(id, user_id, resource_id, start_time, end_time, note) VALUES (&1, $2, $3, $4) RETURNING id",
        )
            .bind(rsvp.user_id.clone())
            .bind(rsvp.resource_id.clone())
            .bind(timespan)
            .bind(rsvp.note.clone())
            // .bind(luckychacha_reservation_abi::ReservationStatus::Pending)
            .fetch_one(&self.pool)
            .await?
            .get(0);

        rsvp.id = id;

        Ok(rsvp)
    }

    async fn change_status(&self, id: ReservationId) -> Result<Reservation, ReservationError> {
        todo!()
    }

    async fn update_note(
        &self,
        id: ReservationId,
        note: String,
    ) -> Result<Reservation, ReservationError> {
        todo!()
    }

    async fn delete(&self, id: ReservationId) -> Result<(), ReservationError> {
        todo!()
    }

    async fn get(&self, query: ReservationQuery) -> Result<Vec<Reservation>, ReservationError> {
        todo!()
    }
}
