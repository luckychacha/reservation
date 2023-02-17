#[path = "../src/test_utils.rs"]
mod test_utils;
use crate::test_utils::TestConfig;

use luckychacha_reservation_abi::{
    reservation_service_client::ReservationServiceClient, Config, Reservation, ReserveRequest,
};
use luckychacha_reservation_service::start_server;
use std::time::Duration;
use tonic::transport::Channel;

#[tokio::test]
async fn grpc_server_should_work() {
    let tconfig = TestConfig::with_server_port(50000);
    let mut client = get_test_client(&tconfig).await;

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
