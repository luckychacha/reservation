mod pb;

use chrono::{DateTime, NaiveDateTime, Utc};
use prost_types::Timestamp;
// 这样在别的地方引用 abi 深层代码的时候就可以直接 abi::xxx 了
pub use pb::*;

pub fn convert_to_utc_time(ts: Timestamp) -> DateTime<Utc> {
    DateTime::<Utc>::from_utc(
        NaiveDateTime::from_timestamp_opt(ts.seconds, ts.nanos as _).unwrap(),
        Utc,
    )
}
