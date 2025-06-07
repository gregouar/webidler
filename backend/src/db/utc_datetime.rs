use sqlx::decode::Decode;
use sqlx::sqlite::{Sqlite, SqliteValueRef};
use sqlx::types::chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::types::Type;
use std::error::Error;

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

impl<'r> Decode<'r, Sqlite> for UtcDateTime {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let naive = <NaiveDateTime as Decode<Sqlite>>::decode(value)?;
        Ok(UtcDateTime(DateTime::<Utc>::from_naive_utc_and_offset(
            naive, Utc,
        )))
    }
}

impl Type<Sqlite> for UtcDateTime {
    fn type_info() -> <Sqlite as sqlx::Database>::TypeInfo {
        <NaiveDateTime as Type<Sqlite>>::type_info()
    }
}
