extern crate hivex_sys;
//
pub mod hive;
//
use chrono::{DateTime, NaiveDateTime, Utc};
//

//
pub fn epoch_to_timestamp(sec: i64) -> DateTime<Utc> {
    DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(sec, 0), Utc)
}
//
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
