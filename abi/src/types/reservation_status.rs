use std::fmt;
use std::fmt::Formatter;

use crate::{ReservationStatus, RsvpStatus};

impl From<RsvpStatus> for ReservationStatus {
    fn from(value: RsvpStatus) -> Self {
        match value {
            RsvpStatus::Unknown => ReservationStatus::Unknown,
            RsvpStatus::Pending => ReservationStatus::Pending,
            RsvpStatus::Confirmed => ReservationStatus::Confirmed,
            RsvpStatus::Blocked => ReservationStatus::Blocked,
        }
    }
}

impl fmt::Display for ReservationStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ReservationStatus::Unknown => write!(f, "unknown"),
            ReservationStatus::Pending => write!(f, "pending"),
            ReservationStatus::Confirmed => write!(f, "confirmed"),
            ReservationStatus::Blocked => write!(f, "blocked"),
        }
    }
}
