mod conflict;

use sqlx::postgres::PgDatabaseError;

pub use conflict::{ReservationConflictInfo, ReservationWindow};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Database error")]
    DbError(sqlx::Error),

    #[error("Reservation Conflict Error.")]
    ConflictReservation(ReservationConflictInfo),

    #[error("No reservation found by the given query condition to confirm.")]
    ReservationNotFound,

    #[error("Invalid reservation id: {0}")]
    InvalidReservationId(i64),

    #[error("Invalid start or end time for the reservation")]
    InvalidTime,

    #[error("Invalid user id: {0}")]
    InvalidUserId(String),

    #[error("Invalid resource id: {0}")]
    InvalidResourceId(String),

    #[error("unknown error")]
    Unknown,
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Error::DbError(_), Error::DbError(_)) => true,
            (Error::ConflictReservation(v1), Error::ConflictReservation(v2)) => v1 == v2,
            (Error::ReservationNotFound, Error::ReservationNotFound) => true,
            (Error::InvalidReservationId(v1), Error::InvalidReservationId(v2)) => v1 == v2,
            (Error::InvalidTime, Error::InvalidTime) => true,
            (Error::InvalidUserId(v1), Error::InvalidUserId(v2)) => v1 == v2,
            (Error::InvalidResourceId(v1), Error::InvalidResourceId(v2)) => v1 == v2,
            (Error::Unknown, Error::Unknown) => true,
            _ => false,
        }
    }
}

impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::Database(e) => {
                let err: &PgDatabaseError = e.downcast_ref();
                match (err.code(), err.schema(), err.table()) {
                    ("23P01", Some("rsvp"), Some("reservation")) => {
                        Error::ConflictReservation(err.detail().unwrap().parse().unwrap())
                    }
                    _ => Error::DbError(sqlx::Error::Database(e)),
                }
            }
            sqlx::Error::RowNotFound => Error::ReservationNotFound,
            _ => Error::DbError(e),
        }
    }
}
