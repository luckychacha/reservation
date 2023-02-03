use crate::{ReservationId, ReservationManager, Rsvp};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use luckychacha_reservation_abi::{Error, Reservation};
use luckychacha_reservation_abi::{FilterPager, Validator};
use sqlx::postgres::types::PgRange;
use sqlx::PgPool;
use sqlx::Row;

#[async_trait]
impl Rsvp for ReservationManager {
    async fn reserve(&self, mut rsvp: Reservation) -> Result<Reservation, Error> {
        rsvp.validate()?;

        let status = luckychacha_reservation_abi::ReservationStatus::from_i32(rsvp.status)
            .unwrap_or(luckychacha_reservation_abi::ReservationStatus::Pending);

        let timespan: PgRange<DateTime<Utc>> = rsvp.get_timespan();

        let id = sqlx::query(
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

        rsvp.id = id;

        Ok(rsvp)
    }

    async fn change_status(&self, id: ReservationId) -> Result<Reservation, Error> {
        id.validate()?;
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
        id.validate()?;
        let rsvp: luckychacha_reservation_abi::Reservation = sqlx::query_as(
            "
                UPDATE rsvp.reservation
                    SET note = $1
                WHERE id = $2
                RETURNING *
            ",
        )
        .bind(note)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(rsvp)
    }

    async fn delete(&self, id: ReservationId) -> Result<(), Error> {
        id.validate()?;
        sqlx::query("DELETE FROM rsvp.reservation WHERE id= $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get(&self, id: ReservationId) -> Result<Reservation, Error> {
        id.validate()?;
        let rsvp: luckychacha_reservation_abi::Reservation = sqlx::query_as(
            "
            SELECT * FROM rsvp.reservation WHERE id = $1
            ",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(rsvp)
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

    async fn filter(
        &self,
        filter: luckychacha_reservation_abi::ReservationFilter,
    ) -> Result<
        (FilterPager, Vec<luckychacha_reservation_abi::Reservation>),
        luckychacha_reservation_abi::Error,
    > {
        let user_id = str_to_option(&filter.user_id);
        let resource_id = str_to_option(&filter.resource_id);
        let status = luckychacha_reservation_abi::ReservationStatus::from_i32(filter.status)
            .unwrap_or(luckychacha_reservation_abi::ReservationStatus::Pending);

        let page_size = if filter.page_size < 10 || filter.page_size > 100 {
            10
        } else {
            filter.page_size
        };
        let rsvp_rows: Vec<Reservation> = sqlx::query_as(
            "SELECT * FROM rsvp.filter($1, $2, $3::rsvp.reservation_status, $4, $5, $6)",
        )
        .bind(user_id)
        .bind(resource_id)
        .bind(status.to_string())
        .bind(filter.cursor)
        .bind(filter.desc)
        .bind(page_size)
        .fetch_all(&self.pool)
        .await?;

        let has_prev = !rsvp_rows.is_empty() && filter.cursor == Some(rsvp_rows[0].id);
        let start = if has_prev { 1 } else { 0 };
        // let start_id = rsvp_rows[start].id;
        let has_next = !rsvp_rows.is_empty() && rsvp_rows.len() - start > page_size as usize;
        let end = if has_next {
            // rsvp_rows[rsvp_rows.len() - 1].id
            rsvp_rows.len() - 1
        } else {
            rsvp_rows.len()
        };
        // let end_id = rsvp_rows[end].id;

        // TODO: optimize this clone.
        let result = rsvp_rows[start..end].to_vec();

        let prev = if has_prev {
            Some(rsvp_rows[0].id)
        } else {
            None
        };

        let next = if has_next {
            Some(rsvp_rows[end].id)
        } else {
            None
        };

        let pager = FilterPager {
            next,
            prev,
            // TODO: How to get total efficiently?
            total: Some(0),
        };
        Ok((pager, result))
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
        Reservation, ReservationConflictInfo, ReservationFilterBuilder, ReservationQueryBuilder,
    };
    use prost_types::Timestamp;

    use super::*;

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_should_work_for_valid_window() {
        let (_manager, rsvp) = make_luckychacha_reservation(migrated_pool.clone()).await;
        assert_ne!(rsvp.id, 0);
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
        println!("rsvp: {rsvp:?}");
        assert!(rsvp.id > 0);

        let rsvp = manager.change_status(rsvp.id).await.unwrap();

        assert_eq!(
            rsvp.status,
            luckychacha_reservation_abi::ReservationStatus::Confirmed as i32
        );
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reservation_change_status_twice_should_do_nothing() {
        let (manager, rsvp) = make_alice_reservation(migrated_pool.clone()).await;

        assert!(rsvp.id > 0);

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
        assert!(rsvp.id > 0);

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
        assert!(rsvp.id > 0);

        let rsvp1 = manager.get(rsvp.id).await.unwrap();

        assert_eq!(rsvp, rsvp1);
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn delete_reservation_should_work() {
        let (manager, rsvp) = make_alice_reservation(migrated_pool.clone()).await;
        assert!(rsvp.id > 0);

        manager.delete(rsvp.id).await.unwrap();

        let get_return_err = manager.get(rsvp.id).await.unwrap_err();

        assert_eq!(get_return_err, Error::ReservationNotFound);
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn query_reservations_should_work() {
        let (manager, rsvp) = make_alice_reservation(migrated_pool.clone()).await;
        assert!(rsvp.id > 0);

        let query = ReservationQueryBuilder::default()
            .user_id("alice")
            .start("2022-12-25T15:00:00+0800".parse::<Timestamp>().unwrap())
            .end("2022-12-28T11:00:00+0800".parse::<Timestamp>().unwrap())
            .status(luckychacha_reservation_abi::ReservationStatus::Pending as i32)
            .build()
            .unwrap();
        let rsvps = manager.query(query).await.unwrap();
        assert_eq!(rsvps.len(), 1);
        assert_eq!(rsvps[0], rsvp);

        // if window is not in range, should return empty
        let query = ReservationQueryBuilder::default()
            .user_id("alice")
            .start("2022-12-29T15:00:00+0800".parse::<Timestamp>().unwrap())
            .end("2022-12-30T11:00:00+0800".parse::<Timestamp>().unwrap())
            .status(luckychacha_reservation_abi::ReservationStatus::Pending as i32)
            .build()
            .unwrap();
        let rsvps = manager.query(query).await.unwrap();
        assert_eq!(rsvps.len(), 0);

        // if status is not in correct, should return empty
        let query = ReservationQueryBuilder::default()
            .user_id("alice")
            .start("2022-12-25T15:00:00+0800".parse::<Timestamp>().unwrap())
            .end("2022-12-28T11:00:00+0800".parse::<Timestamp>().unwrap())
            .status(luckychacha_reservation_abi::ReservationStatus::Confirmed as i32)
            .build()
            .unwrap();
        let rsvps = manager.query(query).await.unwrap();
        assert_eq!(rsvps.len(), 0);

        // change state to confirmed
        let rsvp = manager.change_status(rsvp.id).await.unwrap();
        let query = ReservationQueryBuilder::default()
            .user_id("alice")
            .start("2022-12-25T15:00:00+0800".parse::<Timestamp>().unwrap())
            .end("2022-12-28T11:00:00+0800".parse::<Timestamp>().unwrap())
            .status(luckychacha_reservation_abi::ReservationStatus::Confirmed as i32)
            .build()
            .unwrap();
        let rsvps = manager.query(query).await.unwrap();
        assert_eq!(rsvps.len(), 1);
        assert_eq!(rsvps[0], rsvp);
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn filter_reservations_should_work() {
        let (manager, rsvp) = make_alice_reservation(migrated_pool.clone()).await;
        assert!(rsvp.id > 0);

        let filter = ReservationFilterBuilder::default()
            .user_id("alice")
            .status(luckychacha_reservation_abi::ReservationStatus::Pending as i32)
            .build()
            .unwrap();
        let (pager, rsvps) = manager.filter(filter).await.unwrap();
        assert_eq!(pager.prev, None);
        assert_eq!(pager.next, None);

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
