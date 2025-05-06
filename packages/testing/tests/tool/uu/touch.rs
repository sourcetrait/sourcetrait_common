#[cfg(test)]
mod tests {
    use chrono::{DateTime, TimeZone, Utc};
    use asmov_common_testing::prelude::*;

    static TESTING: testing::Module = testing::module!(Integration, {
        .using_temp_dir()
    });

    fn test_time() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2025, 1, 2, 0, 0, 0).unwrap()
    }

    #[tested]
    fn test_touch_timestamp() {
        let test = testing::test!({
            .using_temp_dir()
        });

        let temp_file = test.temp_dir().join("touch.txt");
        let test_time = test_time();

        tooling::touch_file(&temp_file, Some(test_time));
        assert!(temp_file.exists());

        let filemeta = std::fs::metadata(temp_file).unwrap();
        let modified: DateTime<Utc> = filemeta.modified().unwrap().into();
        assert_eq!(test_time, modified);
    }
}
