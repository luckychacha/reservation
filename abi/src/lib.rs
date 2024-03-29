mod config;
mod error;
mod pager;
mod pb;
mod types;
mod utils;

// 这样在别的地方引用 abi 深层代码的时候就可以直接 abi::xxx 了
pub use config::*;
pub use error::{Error, ReservationConflictInfo, ReservationWindow};
pub use pb::*;

pub use utils::*;

pub type ReservationId = i64;
// pub type UserId = String;
// pub type ResourceId = String;

pub trait Validator {
    fn validate(&self) -> Result<(), Error>;
}

pub trait Normalizer: Validator {
    fn normalize(&mut self) -> Result<(), Error> {
        self.validate()?;
        self.do_normalize();
        Ok(())
    }

    fn do_normalize(&mut self);
}

pub trait ToSql {
    fn to_sql(&self) -> String;
}

/// database equivalent of the "resevation_status" enum.
#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "reservation_status", rename_all = "lowercase")]
pub enum RsvpStatus {
    Unknown,
    Pending,
    Confirmed,
    Blocked,
}

impl Validator for ReservationId {
    fn validate(&self) -> Result<(), Error> {
        if *self <= 0 {
            Err(Error::InvalidReservationId(*self))
        } else {
            Ok(())
        }
    }
}

// impl Validator for String {
//     fn validate(&self) -> Result<(), Error> {
//         if self.is_empty() {
//             Err(Error::InvalidUserOrResourceId(self.clone()))
//         } else {
//             Ok(())
//         }
//     }
// }

// impl Validator for ResourceId {
//     fn validate(&self) -> Result<(), Error> {
//         if self.is_empty() {
//             Err(Error::InvalidResourceId(self.clone()))
//         } else {
//             Ok(())
//         }
//     }
// }
