// 1.add dependency
// cargo add sqlx --features chrono,uuid -p luckychacha-reservation
// cargo add sqlx --features runtime-tokio-rustls,postgres -p luckychacha-reservation
// cargo add sqlx -p luckychacha-reservation

// 2.install sqlx-cli
// cargo install sqlx-cli
// sqlx migrate add init -r

// 3.init pgconfig
// touch .env
// DATABASE_URL = 'postgres://username@host:port/reservation'

// rm /usr/local/var/postgresql@11/postmaster.pid
// brew services restart postgresql

// 4.run xxx.up.sql
// sqlx migrate run
