// pub fn convert_windows_ticks(epoch: i64) -> Result<DateTime<chrono::Utc>> {
//     // Create NaiveDateTime
//     let ndt = chrono::NaiveDateTime::from_timestamp(epoch, 0);
//     Ok(DateTime::from_utc(ndt, chrono::Utc))
// }

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
