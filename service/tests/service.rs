#[path = "../src/test_utils.rs"]
mod test_utils;
use crate::test_utils::TestConfig;

use futures::StreamExt;
use luckychacha_reservation_abi::{
    reservation_service_client::ReservationServiceClient, Config, FilterRequest, FilterResponse,
    QueryRequest, Reservation, ReservationFilterBuilder, ReservationQueryBuilder, ReserveRequest,
};
use luckychacha_reservation_service::start_server;
use std::time::Duration;
use tonic::transport::Channel;

#[tokio::test]
async fn grpc_server_should_work() {
    let tconfig = TestConfig::with_server_port(50000);
    let mut client = get_test_client(&tconfig).await;

    // 1.make a reservation
    let mut rsvp = Reservation::new_pending(
        "luckychacha-id",
        "ocean-view-room-666",
        "2022-12-25T15:00:00+0800".parse().unwrap(),
        "2022-12-28T11:00:00+0800".parse().unwrap(),
        String::from("I'll arrive at 3pm. Please help to upgrade to execuitive room if possible."),
    );
    let request = ReserveRequest::new(rsvp.clone());
    let ret: Reservation = client
        .reserve(request)
        .await
        .unwrap()
        .into_inner()
        .reservation
        .unwrap();

    rsvp.id = ret.id;
    assert_eq!(ret, rsvp);

    // 2.make another reservation with the same resource and has time conflict with last reservation
    let rsvp2 = Reservation::new_pending(
        "luckychacha-id",
        "ocean-view-room-666",
        "2022-12-25T15:00:00+0800".parse().unwrap(),
        "2022-12-28T11:00:00+0800".parse().unwrap(),
        String::from("I'll arrive at 3pm. Please help to upgrade to execuitive room if possible."),
    );
    let request = ReserveRequest::new(rsvp2.clone());
    let ret = client.reserve(request).await;
    assert!(ret.is_err());

    // 4.query grpc interface test.

    // 5.filter grpc interface test.
}

#[tokio::test]
async fn grpc_query_should_work() {
    let tconfig = TestConfig::with_server_port(50001);
    let mut client = get_test_client(&tconfig).await;

    // 1.make some reservations.
    make_reservations(&mut client, 100).await;

    let query = ReservationQueryBuilder::default()
        .user_id("luckychacha-id")
        .build()
        .unwrap();

    let mut ret = client
        .query(QueryRequest::new(query))
        .await
        .unwrap()
        .into_inner();
    while let Some(Ok(ret)) = ret.next().await {
        assert_eq!(ret.user_id, "luckychacha-id");
    }
}

#[tokio::test]
async fn grpc_filter_should_work() {
    let config = TestConfig::with_server_port(50002);
    let mut client = get_test_client(&config).await;

    // 1.make some reservations
    make_reservations(&mut client, 25).await;

    let filter = ReservationFilterBuilder::default()
        .user_id("luckychacha-id")
        .build()
        .unwrap();

    let FilterResponse {
        reservations,
        pager,
    } = client
        .filter(FilterRequest::new(filter.clone()))
        .await
        .unwrap()
        .into_inner();

    let pager = pager.unwrap();
    assert_eq!(pager.next, Some(filter.page_size));
    assert_eq!(pager.prev, None);
    assert_eq!(pager.total, None);
    assert_eq!(reservations.len(), filter.page_size as usize);

    let filter = filter.next_page(&pager).unwrap();
    // then we get next page
    let FilterResponse {
        pager,
        reservations,
    } = client
        .filter(FilterRequest::new(filter.clone()))
        .await
        .unwrap()
        .into_inner();

    let pager = pager.unwrap();

    assert_eq!(pager.next, filter.cursor.map(|v| v + filter.page_size));
    assert_eq!(pager.prev, filter.cursor.map(|v| v + 1));
    assert_eq!(reservations.len(), filter.page_size as usize);
}

async fn get_test_client(config: &Config) -> ReservationServiceClient<Channel> {
    let config_clone = config.clone();

    tokio::spawn(async move {
        start_server(&config_clone).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    ReservationServiceClient::connect(config.server.server_url(false))
        .await
        .unwrap()
}

async fn make_reservations(client: &mut ReservationServiceClient<Channel>, n: usize) {
    for i in 0..n {
        let rsvp = Reservation::new_pending(
            "luckychacha-id",
            format!("ocean-view-room-{i}"),
            "2022-12-25T15:00:00+0800".parse().unwrap(),
            "2022-12-28T11:00:00+0800".parse().unwrap(),
            String::from("Test notes."),
        );
        client
            .reserve(ReserveRequest::new(rsvp))
            .await
            .unwrap()
            .into_inner()
            .reservation
            .unwrap();
    }
}
