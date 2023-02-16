use futures::Stream;
use luckychacha_reservation::{ReservationManager, Rsvp};
use luckychacha_reservation_abi::{
    reservation_service_server::ReservationService, CancelRequest, CancelResponse, Config,
    ConfirmRequest, ConfirmResponse, FilterRequest, FilterResponse, GetRequest, GetResponse,
    ListenRequest, QueryRequest, ReserveRequest, ReserveResponse, UpdateRequest, UpdateResponse,
};
use tokio::sync::mpsc;
use tonic::{async_trait, Request, Response, Status};

use crate::{ReservationStream, RsvpService, TonicReceiverStream};

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
        request: Request<ConfirmRequest>,
    ) -> Result<Response<ConfirmResponse>, Status> {
        let request = request.into_inner();
        let rsvp = self.manager.change_status(request.id).await?;
        Ok(Response::new(ConfirmResponse {
            reservation: Some(rsvp),
        }))
    }

    async fn update(
        &self,
        request: Request<UpdateRequest>,
    ) -> Result<Response<UpdateResponse>, Status> {
        let request = request.into_inner();
        let rsvp = self.manager.update_note(request.id, request.note).await?;
        Ok(Response::new(UpdateResponse {
            reservation: Some(rsvp),
        }))
    }

    async fn cancel(
        &self,
        request: Request<CancelRequest>,
    ) -> Result<Response<CancelResponse>, Status> {
        let request = request.into_inner();
        let rsvp = self.manager.delete(request.id).await?;
        Ok(Response::new(CancelResponse {
            reservation: Some(rsvp),
        }))
    }

    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        let request = request.into_inner();
        let rsvp = self.manager.get(request.id).await?;
        Ok(Response::new(GetResponse {
            reservation: Some(rsvp),
        }))
    }

    type queryStream = ReservationStream;

    async fn query(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<Self::queryStream>, Status> {
        let request = request.into_inner();
        if request.query.is_none() {
            return Err(Status::invalid_argument("missing query"));
        }
        let rsvps = self.manager.query(request.query.unwrap()).await;
        let stream = TonicReceiverStream::new(rsvps);
        Ok(Response::new(Box::pin(stream)))
    }

    async fn filter(
        &self,
        request: Request<FilterRequest>,
    ) -> Result<Response<FilterResponse>, Status> {
        let request = request.into_inner();
        if request.filter.is_none() {
            return Err(Status::invalid_argument("missing filter"));
        }
        let (pager, rsvp) = self.manager.filter(request.filter.unwrap()).await?;
        Ok(Response::new(FilterResponse {
            reservations: rsvp,
            pager: Some(pager),
        }))
    }

    type listenStream = ReservationStream;

    async fn listen(
        &self,
        _request: Request<ListenRequest>,
    ) -> Result<Response<Self::listenStream>, Status> {
        todo!()
    }
}

impl<T> Stream for TonicReceiverStream<T> {
    type Item = Result<T, Status>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        match self.inner.poll_recv(cx) {
            std::task::Poll::Ready(Some(Ok(item))) => std::task::Poll::Ready(Some(Ok(item))),
            std::task::Poll::Ready(Some(Err(e))) => std::task::Poll::Ready(Some(Err(e.into()))),
            std::task::Poll::Ready(None) => std::task::Poll::Ready(None),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

impl<T> TonicReceiverStream<T> {
    pub fn new(inner: mpsc::Receiver<Result<T, luckychacha_reservation_abi::Error>>) -> Self {
        TonicReceiverStream { inner }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::TestConfig;
    use luckychacha_reservation_abi::Reservation;

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
