mod error;
mod pb;
mod types;
mod utils;

// 这样在别的地方引用 abi 深层代码的时候就可以直接 abi::xxx 了
pub use error::{Error, ReservationConflictInfo, ReservationWindow};
pub use pb::*;

pub use utils::*;

/// database equivalent of the "resevation_status" enum.
#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "reservation_status", rename_all = "lowercase")]
pub enum RsvpStatus {
    Unknown,
    Pending,
    Confirmed,
    Blocked,
}
