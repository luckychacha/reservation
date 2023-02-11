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
    use super::*;
    use luckychacha_reservation_abi::Reservation;

    use luckychacha_sqlx_pg_tester::TestDb;

    #[tokio::test]
    async fn reserve_should_work() {
        let mut config = Config::load("fixtures/config.yml").unwrap();

        let tdb = TestDb::new(
            &config.db.user,
            &config.db.password,
            &config.db.host,
            config.db.port,
            "../migrations",
        );

        config.db.dbname = tdb.dbname.clone();

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
