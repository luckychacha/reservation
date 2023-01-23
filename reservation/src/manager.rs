use std::str::FromStr;

use crate::{ReservationId, ReservationManager, Rsvp};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use luckychacha_reservation_abi::Validator;
use luckychacha_reservation_abi::{Error, Reservation};
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

        let timespan: PgRange<DateTime<Utc>> = rsvp.get_timespan();

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

    async fn change_status(&self, id: ReservationId) -> Result<Reservation, Error> {
        let id = Uuid::parse_str(&id).map_err(|_| Error::InvalidReservationId(id.clone()))?;
        let rsvp: luckychacha_reservation_abi::Reservation = sqlx::query_as(
            "
                UPDATE rsvp.reservation
                    SET status = 'confirmed'
                WHERE id = $1
                    AND status = 'pending'
                RETURNING *
            ",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        Ok(rsvp)
    }

    async fn update_note(&self, id: ReservationId, note: String) -> Result<Reservation, Error> {
        let id = Uuid::from_str(&id).map_err(|_| Error::InvalidReservationId(id.clone()))?;
        let rsvp: luckychacha_reservation_abi::Reservation = sqlx::query_as(
            "
                UPDATE rsvp.reservation
                    SET note = $1
                WHERE id = $2::UUID
                RETURNING *
            ",
        )
        .bind(note)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(rsvp)
    }

    async fn get(&self, id: ReservationId) -> Result<Reservation, Error> {
        let id = Uuid::parse_str(&id).map_err(|_| Error::InvalidReservationId(id.clone()))?;

        let rsvp: luckychacha_reservation_abi::Reservation = sqlx::query_as(
            "
            SELECT * FROM rsvp.reservation WHERE id = $1::UUID
            ",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(rsvp)
    }

    async fn delete(&self, id: ReservationId) -> Result<(), Error> {
        let id = Uuid::parse_str(&id).map_err(|_| Error::InvalidReservationId(id.clone()))?;
        sqlx::query("DELETE FROM rsvp.reservation WHERE id= $1::UUID")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn query(
        &self,
        query: luckychacha_reservation_abi::ReservationQuery,
    ) -> Result<Vec<luckychacha_reservation_abi::Reservation>, luckychacha_reservation_abi::Error>
    {
        let user_id = str_to_option(&query.user_id);
        let resource_id = str_to_option(&query.resource_id);
        let range: PgRange<DateTime<Utc>> = query.get_timepspan();
        let status = luckychacha_reservation_abi::ReservationStatus::from_i32(query.status)
            .unwrap_or(luckychacha_reservation_abi::ReservationStatus::Pending);
        let rsvp_rows = sqlx::query_as(
            "SELECT * FROM rsvp.query($1, $2, $3, $4::rsvp.reservation_status, $5, $6, $7)",
        )
        .bind(user_id)
        .bind(resource_id)
        .bind(range)
        .bind(status.to_string())
        .bind(query.page)
        .bind(query.desc)
        .bind(query.page_size)
        .fetch_all(&self.pool)
        .await?;

        Ok(rsvp_rows)
    }
}

impl ReservationManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn str_to_option(s: &str) -> Option<&str> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

#[cfg(test)]
mod tests {
    use luckychacha_reservation_abi::{
        Reservation, ReservationConflictInfo, ReservationQueryBuilder,
    };
    use prost_types::Timestamp;

    use super::*;

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_should_work_for_valid_window() {
        let (_manager, rsvp) = make_luckychacha_reservation(migrated_pool.clone()).await;
        assert_ne!(rsvp.id, "");
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_conflict_reservation_should_reject() {
        let (manager, _rsvp1) = make_luckychacha_reservation(migrated_pool.clone()).await;

        let rsvp2 = Reservation::new_pending(
            "luckychacha-id",
            "ocean-view-room-666",
            "2022-12-25T15:00:00+0800".parse().unwrap(),
            "2022-12-28T11:00:00+0800".parse().unwrap(),
            String::from(
                "I'll arrive at 3pm. Please help to upgrade to execuitive room if possible.",
            ),
        );

        // let _rsvp1 = manager.reserve(rsvp1).await.unwrap();

        let err = manager.reserve(rsvp2).await.unwrap_err();
        if let Error::ConflictReservation(ReservationConflictInfo::Parsed(info)) = err {
            assert_eq!(info.old.rid, "ocean-view-room-666");
        }
        // assert!(err, Error::Conflict);
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reservation_change_status_should_work() {
        let (manager, rsvp) = make_alice_reservation(migrated_pool.clone()).await;
        println!("rsvp: {:?}", rsvp);
        assert!(!rsvp.id.is_empty());

        let rsvp = manager
            .change_status(rsvp.id.parse().unwrap())
            .await
            .unwrap();

        assert_eq!(
            rsvp.status,
            luckychacha_reservation_abi::ReservationStatus::Confirmed as i32
        );
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reservation_change_status_twice_should_do_nothing() {
        let (manager, rsvp) = make_alice_reservation(migrated_pool.clone()).await;

        assert!(!rsvp.id.is_empty());

        let rsvp = manager.change_status(rsvp.id).await.unwrap();

        let rsvp = manager.change_status(rsvp.id).await.unwrap_err();

        assert_eq!(
            rsvp,
            luckychacha_reservation_abi::Error::ReservationNotFound,
        );
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn update_note_should_work() {
        let (manager, rsvp) = make_alice_reservation(migrated_pool.clone()).await;
        assert!(!rsvp.id.is_empty());

        let rsvp = manager
            .update_note(rsvp.id, "Hello world".into())
            .await
            .unwrap();

        // let rsvp = manager.change_status(rsvp.id).await.unwrap_err();

        assert_eq!(rsvp.note, "Hello world",);
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn get_reservation_should_work() {
        let (manager, rsvp) = make_alice_reservation(migrated_pool.clone()).await;
        assert!(!rsvp.id.is_empty());

        let rsvp1 = manager.get(rsvp.id.clone()).await.unwrap();

        assert_eq!(rsvp, rsvp1);
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn delete_reservation_should_work() {
        let (manager, rsvp) = make_alice_reservation(migrated_pool.clone()).await;
        assert!(!rsvp.id.is_empty());

        manager.delete(rsvp.id.clone()).await.unwrap();

        let get_return_err = manager.get(rsvp.id.clone()).await.unwrap_err();

        assert_eq!(get_return_err, Error::ReservationNotFound);
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn query_reservations_should_work() {
        let (manager, rsvp) = make_alice_reservation(migrated_pool.clone()).await;
        assert!(!rsvp.id.is_empty());

        let query = ReservationQueryBuilder::default()
            .user_id("alice")
            .start("2022-12-25T15:00:00+0800".parse::<Timestamp>().unwrap())
            .end("2022-12-28T11:00:00+0800".parse::<Timestamp>().unwrap())
            .status(luckychacha_reservation_abi::ReservationStatus::Pending as i32)
            // .page(1)
            // .page_size(10)
            // .desc(false)
            .resource_id("ixia-test-1")
            .build()
            .unwrap();
        let rsvps = manager.query(query).await.unwrap();
        assert_eq!(rsvps.len(), 1);
        assert_eq!(rsvps[0], rsvp);
    }

    // luckychacha reservation template
    async fn make_luckychacha_reservation(pool: PgPool) -> (ReservationManager, Reservation) {
        make_reservation(
            pool,
            "luckychacha",
            "ocean-view-room-666",
            "2022-12-25T15:00:00+0800",
            "2022-12-28T11:00:00+0800",
            "I need to book this for xyz project for a month",
        )
        .await
    }

    // alice reservation template
    async fn make_alice_reservation(pool: PgPool) -> (ReservationManager, Reservation) {
        make_reservation(
            pool,
            "alice",
            "ixia-test-1",
            "2022-12-25T15:00:00+0800",
            "2022-12-28T11:00:00+0800",
            "I need to book this for xyz project for a month",
        )
        .await
    }

    async fn make_reservation(
        pool: PgPool,
        uid: &str,
        rid: &str,
        start: &str,
        end: &str,
        note: &str,
    ) -> (ReservationManager, Reservation) {
        let manager = ReservationManager::new(pool.clone());
        let rsvp = luckychacha_reservation_abi::Reservation::new_pending(
            uid,
            rid,
            start.parse().unwrap(),
            end.parse().unwrap(),
            note,
        );
        let rsvp = manager.reserve(rsvp).await.unwrap();
        (manager, rsvp)
    }
}
