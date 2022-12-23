use crate::{ReservationError, ReservationId, ReservationManager, Rsvp};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use luckychacha_reservation_abi::{convert_to_utc_time, Reservation, ReservationQuery};
use sqlx::postgres::types::PgRange;
use sqlx::Row;

#[async_trait]
impl Rsvp for ReservationManager {
    async fn reserve(&self, mut rsvp: Reservation) -> Result<Reservation, ReservationError> {
        if rsvp.start.is_none() || rsvp.end.is_none() {
            return Err(ReservationError::InvalidTime);
        }

        // let status = luckychacha_reservation_abi::ReservationStatus::from_i32(rsvp.status)
        //     .unwrap_or(luckychacha_reservation_abi::ReservationStatus::Pending);
        let start = convert_to_utc_time(rsvp.start.as_ref().unwrap().clone());
        let end = convert_to_utc_time(rsvp.end.as_ref().unwrap().clone());

        let timespan: PgRange<DateTime<Utc>> = (start..end).into();

        let id = sqlx::query(
            "INSERT INTO reservation(id, user_id, resource_id, start_time, end_time, note, status) VALUES ($1, $2, $3, $4, $5) RETURNING id",
        )
            .bind(rsvp.user_id.clone())
            .bind(rsvp.resource_id.clone())
            .bind(timespan)
            .bind(rsvp.note.clone())
            .bind(rsvp.status)
            .fetch_one(&self.pool)
            .await?
            .get(0);

        rsvp.id = id;

        Ok(rsvp)
    }

    async fn change_status(&self, _id: ReservationId) -> Result<Reservation, ReservationError> {
        todo!()
    }

    async fn update_note(
        &self,
        _id: ReservationId,
        _note: String,
    ) -> Result<Reservation, ReservationError> {
        todo!()
    }

    async fn delete(&self, _id: ReservationId) -> Result<(), ReservationError> {
        todo!()
    }

    async fn get(&self, _query: ReservationQuery) -> Result<Vec<Reservation>, ReservationError> {
        todo!()
    }
}
