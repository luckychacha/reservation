use crate::{ReservationId, ReservationManager, Rsvp};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use luckychacha_reservation_abi::{Error, Reservation, ReservationQuery};
use sqlx::postgres::types::PgRange;
use sqlx::types::Uuid;
use sqlx::PgPool;
use sqlx::Row;

#[async_trait]
impl Rsvp for ReservationManager {
    async fn reserve(&self, mut rsvp: Reservation) -> Result<Reservation, Error> {
        rsvp.validate()?;

        let status = luckychacha_reservation_abi::ReservationStatus::from_i32(rsvp.status)
            .unwrap_or(luckychacha_reservation_abi::ReservationStatus::Pending);

        let timespan: PgRange<DateTime<Utc>> = rsvp.get_timespan().into();

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

    async fn change_status(&self, _id: ReservationId) -> Result<Reservation, Error> {
        todo!()
    }

    async fn update_note(&self, _id: ReservationId, _note: String) -> Result<Reservation, Error> {
        todo!()
    }

    async fn delete(&self, _id: ReservationId) -> Result<(), Error> {
        todo!()
    }

    async fn get(&self, _query: ReservationQuery) -> Result<Vec<Reservation>, Error> {
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
    use luckychacha_reservation_abi::ReservationConflictInfo;

    use super::*;

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_should_work_for_valid_window() {
        let manager = ReservationManager::new(migrated_pool.clone());
        let rsvp = Reservation::new_pending(
            "luckychacha-id",
            "ocean-view-room-666",
            "2022-12-25T15:00:00+0800".parse().unwrap(),
            "2022-12-28T11:00:00+0800".parse().unwrap(),
            String::from(
                "I'll arrive at 3pm. Please help to upgrade to execuitive room if possible.",
            ),
        );

        let rsvp = manager.reserve(rsvp).await.unwrap();
        assert_ne!(rsvp.id, "");
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_conflict_reservation_should_reject() {
        let manager = ReservationManager::new(migrated_pool.clone());

        let rsvp1 = Reservation::new_pending(
            "luckychacha-id",
            "ocean-view-room-666",
            "2022-12-25T15:00:00+0800".parse().unwrap(),
            "2022-12-28T11:00:00+0800".parse().unwrap(),
            String::from(
                "I'll arrive at 3pm. Please help to upgrade to execuitive room if possible.",
            ),
        );

        let rsvp2 = Reservation::new_pending(
            "luckychacha-id",
            "ocean-view-room-666",
            "2022-12-25T15:00:00+0800".parse().unwrap(),
            "2022-12-28T11:00:00+0800".parse().unwrap(),
            String::from(
                "I'll arrive at 3pm. Please help to upgrade to execuitive room if possible.",
            ),
        );

        let _rsvp1 = manager.reserve(rsvp1).await.unwrap();

        let err = manager.reserve(rsvp2).await.unwrap_err();
        if let Error::ConflictReservation(ReservationConflictInfo::Parsed(info)) = err {
            assert_eq!(info.old.rid, "ocean-view-room-666");
        }
        // assert!(err, Error::Conflict);
    }
}
