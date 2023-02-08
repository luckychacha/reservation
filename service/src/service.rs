use luckychacha_reservation::{ReservationManager, Rsvp};
use luckychacha_reservation_abi::{
    reservation_service_server::ReservationService, CancelRequest, CancelResponse, Config,
    ConfirmRequest, ConfirmResponse, FilterRequest, FilterResponse, GetRequest, GetResponse,
    ListenRequest, QueryRequest, ReserveRequest, ReserveResponse, UpdateRequest, UpdateResponse,
};
use tonic::{async_trait, Request, Response, Status};

use crate::{ReservationStream, RsvpService};

impl RsvpService {
    pub async fn from_config(config: &Config) -> Result<Self, anyhow::Error> {
        Ok(Self {
            manager: ReservationManager::from_config(&config.db).await?,
        })
    }
}

#[async_trait]
impl ReservationService for RsvpService {
    async fn reserve(
        &self,
        request: Request<ReserveRequest>,
    ) -> Result<Response<ReserveResponse>, Status> {
        let request = request.into_inner();
        if request.reservation.is_none() {
            return Err(Status::invalid_argument("missing reservation"));
        }
        let reservation = self.manager.reserve(request.reservation.unwrap()).await?;

        Ok(Response::new(ReserveResponse {
            reservation: Some(reservation),
        }))
    }

    async fn confirm(
        &self,
        _request: Request<ConfirmRequest>,
    ) -> Result<Response<ConfirmResponse>, Status> {
        todo!()
    }

    async fn update(
        &self,
        _request: Request<UpdateRequest>,
    ) -> Result<Response<UpdateResponse>, Status> {
        todo!()
    }

    async fn cancel(
        &self,
        _request: Request<CancelRequest>,
    ) -> Result<Response<CancelResponse>, Status> {
        todo!()
    }

    async fn get(&self, _request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        todo!()
    }

    type queryStream = ReservationStream;

    async fn query(
        &self,
        _request: Request<QueryRequest>,
    ) -> Result<Response<Self::queryStream>, Status> {
        todo!()
    }

    async fn filter(
        &self,
        _request: Request<FilterRequest>,
    ) -> Result<Response<FilterResponse>, Status> {
        todo!()
    }

    type listenStream = ReservationStream;

    async fn listen(
        &self,
        _request: Request<ListenRequest>,
    ) -> Result<Response<Self::listenStream>, Status> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::{ops::Deref, sync::Arc, thread};

    use lazy_static::lazy_static;
    use luckychacha_reservation_abi::Reservation;
    use sqlx::{types::Uuid, Connection, Executor, PgConnection};
    use tokio::runtime::Runtime;

    use super::*;

    lazy_static! {
        static ref TEST_RT: Runtime = Runtime::new().unwrap();
    }

    struct TestConfig {
        config: Arc<Config>,
    }

    impl Deref for TestConfig {
        type Target = Config;

        fn deref(&self) -> &Self::Target {
            &self.config
        }
    }

    impl TestConfig {
        pub fn new() -> Self {
            let mut config =
                Config::load(shellexpand::tilde("~/.config/reservation.yml").as_ref()).unwrap();
            // let mut config = Config::load("../service/fixtures/config.yml").unwrap();

            // create tmp database
            let rand_uuid = Uuid::new_v4();
            let db_name = format!("test_reservation_{rand_uuid}");
            config.db.dbname = db_name.clone();
            let server_url = config.db.server_url();
            let db_url = config.db.db_url();

            thread::spawn(move || {
                TEST_RT.block_on(async move {
                    let mut conn = PgConnection::connect(&server_url).await.unwrap();
                    conn.execute(format!(r#"CREATE DATABASE "{db_name}""#).as_str())
                        .await
                        .expect("Failed when create database {db_name}.");

                    let mut conn = PgConnection::connect(&db_url).await.unwrap();

                    sqlx::migrate!("../migrations")
                        .run(&mut conn)
                        .await
                        .expect("Failed when migrate.");
                })
            })
            .join()
            .expect("Failed to create database.");
            Self {
                config: Arc::new(config),
            }
        }
    }

    impl Drop for TestConfig {
        fn drop(&mut self) {
            let server_url = self.config.db.server_url();
            let database_name = self.config.db.dbname.clone();
            thread::spawn(move || {
                TEST_RT.block_on(async move {
                    let mut conn = PgConnection::connect(&server_url).await.unwrap();

                    #[allow(clippy::expect_used)]
                    sqlx::query(&format!(r#"SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE pid <> pg_backend_pid() AND datname = '{database_name}'"#))
                        .execute(&mut conn)
                        .await
                        .expect("Terminate all other connections");
                    #[allow(clippy::expect_used)]
                    sqlx::query(&format!(r#"DROP DATABASE "{database_name}""#))
                        .execute(&mut conn)
                        .await
                        .expect("Deleting the database");
                })
            });
        }
    }

    #[tokio::test]
    async fn reserve_should_work() {
        let config = TestConfig::new();
        let service = RsvpService::from_config(&config).await.unwrap();

        let reservation: Reservation = Reservation::new_pending(
            "luckychacha",
            "ocean-view",
            "2023-01-01 14:00:00+0800".parse().unwrap(),
            "2023-01-05 14:00:00+0800".parse().unwrap(),
            "test a note",
        );
        let request: Request<ReserveRequest> = Request::new(ReserveRequest {
            reservation: Some(reservation),
        });
        let res = service.reserve(request).await.unwrap();

        assert!(res.into_inner().reservation.is_some());
    }
}
