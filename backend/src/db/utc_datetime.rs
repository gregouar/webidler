use sqlx::decode::Decode;
use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef};
use sqlx::sqlite::{Sqlite, SqliteValueRef};
use sqlx::types::Type;
use sqlx::types::chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::{Encode, Postgres};
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

impl Type<Sqlite> for UtcDateTime {
    fn type_info() -> <Sqlite as sqlx::Database>::TypeInfo {
        <NaiveDateTime as Type<Sqlite>>::type_info()
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

impl<'q> Encode<'q, Sqlite> for UtcDateTime {
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, Box<dyn Error + Send + Sync>> {
        let naive = self.0.naive_utc();
        <NaiveDateTime as Encode<Sqlite>>::encode_by_ref(&naive, buf)
    }
}

// impl Type<Postgres> for UtcDateTime {
//     fn type_info() -> PgTypeInfo {
//         <DateTime<Utc> as Type<Postgres>>::type_info()
//     }
// }

impl sqlx::Type<Postgres> for UtcDateTime {
    fn type_info() -> PgTypeInfo {
        <DateTime<Utc> as sqlx::Type<Postgres>>::type_info()
    }
}

impl sqlx::postgres::PgHasArrayType for UtcDateTime {
    fn array_type_info() -> PgTypeInfo {
        <DateTime<Utc> as sqlx::postgres::PgHasArrayType>::array_type_info()
    }
}

impl<'r> Decode<'r, Postgres> for UtcDateTime {
    fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let dt = <DateTime<Utc> as Decode<Postgres>>::decode(value)?;
        Ok(UtcDateTime(dt))
    }
}

impl<'q> Encode<'q, Postgres> for UtcDateTime {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, Box<dyn Error + Send + Sync>> {
        <DateTime<Utc> as Encode<Postgres>>::encode_by_ref(&self.0, buf)
    }
}
