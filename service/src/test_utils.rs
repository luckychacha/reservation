use luckychacha_reservation_abi::Config;
use luckychacha_sqlx_pg_tester::TestDb;
use std::ops::Deref;

pub struct TestConfig {
    #[allow(dead_code)]
    tdb: TestDb,
    pub config: Config,
}

impl TestConfig {
    pub fn new() -> Self {
        let mut config = Config::load("fixtures/config.yml").unwrap();

        let tdb = TestDb::new(
            &config.db.user,
            &config.db.password,
            &config.db.host,
            config.db.port,
            "../migrations",
        );

        config.db.dbname = tdb.dbname.clone();
        Self { tdb, config }
    }
}

impl Deref for TestConfig {
    type Target = Config;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}
