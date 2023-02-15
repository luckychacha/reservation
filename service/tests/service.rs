use luckychacha_reservation_abi::Config;
use luckychacha_reservation_service::start_server;

#[tokio::test]
async fn grpc_server_should_work() {
    tokio::spawn(async move {
        let config = Config::load("../fixture/config.yml").unwrap();
        start_server(&config).await.unwrap();
    });
}
