use std::path::Path;

use anyhow::Result;
use luckychacha_reservation_abi::Config;
use luckychacha_reservation_service::start_server;

#[tokio::main]
async fn main() -> Result<()> {
    let filename = std::env::var("RESERVATION_CONFIG").unwrap_or_else(|_| {
        let p1 = Path::new("./reservation.yml");
        let p2 = Path::new("/etc/reservation.yml");
        let tmp = shellexpand::tilde("~/.config/reservation.yml");
        let p3 = Path::new(tmp.as_ref());

        match (p1.exists(), p2.exists(), p3.exists()) {
            (true, _, _) => p1.to_str().unwrap().to_string(),
            (_, true, _) => p2.to_str().unwrap().to_string(),
            (_, _, true) => p3.to_str().unwrap().to_string(),
            _ => panic!("config file not found"),
        }
    });

    let config = Config::load(filename)?;
    println!("{config:?}");

    start_server(&config).await
}
