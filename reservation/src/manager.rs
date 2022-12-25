use crate::{ReservationError, ReservationId, ReservationManager, Rsvp};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use luckychacha_reservation_abi::{convert_to_utc_time, Reservation, ReservationQuery};
use sqlx::postgres::types::PgRange;
use sqlx::types::Uuid;
use sqlx::PgPool;
use sqlx::Row;

#[async_trait]
impl Rsvp for ReservationManager {
    async fn reserve(&self, mut rsvp: Reservation) -> Result<Reservation, ReservationError> {
        if rsvp.start.is_none() || rsvp.end.is_none() {
            return Err(ReservationError::InvalidTime);
        }

        let status = luckychacha_reservation_abi::ReservationStatus::from_i32(rsvp.status)
            .unwrap_or(luckychacha_reservation_abi::ReservationStatus::Pending);
        let start = convert_to_utc_time(rsvp.start.as_ref().unwrap().clone());
        let end = convert_to_utc_time(rsvp.end.as_ref().unwrap().clone());

        let timespan: PgRange<DateTime<Utc>> = (start..end).into();

        let id: Uuid = sqlx::query(
            "INSERT INTO rsvp.reservation(user_id, resource_id, timespan, note, status) VALUES ($1, $2, $3, $4, $5::rsvp.reservation_status) RETURNING id",
        )
            .bind(rsvp.user_id.clone())
            .bind(rsvp.resource_id.clone())
            .bind(timespan)
            .bind(rsvp.note.clone())
            .bind(status.to_string())
            .fetch_one(&self.pool)
            .await?
            .get(0);

        rsvp.id = id.to_string();

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

impl ReservationManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::FixedOffset;
    use luckychacha_reservation_abi::convert_to_timestamp;

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_should_work_for_valid_window() {
        let manager = ReservationManager::new(migrated_pool.clone());
        let start: DateTime<FixedOffset> = "2022-12-25T15:00:00+0800".parse().unwrap();
        let end: DateTime<FixedOffset> = "2022-12-28T11:00:00+0800".parse().unwrap();
        let rsvp = luckychacha_reservation_abi::Reservation {
            id: String::from(""),
            user_id: String::from("luckychacha"),
            resource_id: String::from("ocean-view-room-666"),
            start: Some(convert_to_timestamp(start.with_timezone(&Utc))),
            end: Some(convert_to_timestamp(end.with_timezone(&Utc))),
            note: String::from(
                "I'll arrive at 3pm. Please help to upgrade to execuitive room if possible.",
            ),
            status: luckychacha_reservation_abi::ReservationStatus::Pending as i32,
        };

        let rsvp = manager.reserve(rsvp).await.unwrap();
        assert_ne!(rsvp.id, "");
    }
}
