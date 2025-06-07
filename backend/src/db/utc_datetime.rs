use sqlx::types::chrono::{DateTime, NaiveDateTime, Utc};

#[derive(Debug, Clone)]
pub struct UtcDateTime(DateTime<Utc>);

impl From<NaiveDateTime> for UtcDateTime {
    fn from(naive: NaiveDateTime) -> Self {
        UtcDateTime(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc))
    }
}

impl From<DateTime<Utc>> for UtcDateTime {
    fn from(dt: DateTime<Utc>) -> Self {
        UtcDateTime(dt)
    }
}

impl From<UtcDateTime> for DateTime<Utc> {
    fn from(dt: UtcDateTime) -> Self {
        dt.0
    }
}
