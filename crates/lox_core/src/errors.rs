use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoxError {
    #[error("invalid date `{0}-{1}-{2}`")]
    InvalidDate(i64, i64, i64),
    #[error("invalid time `{0}:{1}:{2}`")]
    InvalidTime(i64, i64, i64),
    #[error("invalid time `{0}:{1}:{2}`")]
    InvalidSeconds(i64, i64, f64),
    #[error("day of year cannot be 366 for a non-leap year")]
    NonLeapYear,
}
