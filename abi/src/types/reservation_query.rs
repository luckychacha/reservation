use chrono::{DateTime, Utc};
use sqlx::postgres::types::PgRange;

use crate::{ReservationQuery, Validator};

use super::{get_timespan, validate_range};

impl ReservationQuery {
    // pub fn new(
    //     resource_id: impl Into<String>,
    //     user_id: impl Into<String>,
    //     start: DateTime<Utc>,
    //     end: DateTime<Utc>,
    //     page: i32,
    //     page_size: i32,
    //     desc: bool,
    //     status: ReservationStatus,
    // ) -> Self {
    //     Self {
    //         resource_id: resource_id.into(),
    //         user_id: user_id.into(),
    //         start: Some(convert_to_timestamp(start)),
    //         end: Some(convert_to_timestamp(end)),
    //         status: status as i32,
    //         page,
    //         page_size,
    //         desc,
    //     }
    // }

    pub fn get_timepspan(&self) -> PgRange<DateTime<Utc>> {
        get_timespan(self.start.as_ref(), self.end.as_ref())
    }
}

impl Validator for ReservationQuery {
    fn validate(&self) -> Result<(), crate::Error> {
        validate_range(self.start.as_ref(), self.end.as_ref())
    }
}
