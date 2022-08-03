use std::fmt;

use chrono::NaiveDateTime;

use serde::{ Deserialize, Deserializer };
use serde::ser::{ SerializeStruct, Serializer, Serialize };
use serde::de::{ self, Visitor };


pub struct SerializableDate{ timestamp: NaiveDateTime }

impl From<NaiveDateTime> for SerializableDate {
  fn from(dt: NaiveDateTime) -> Self {
    SerializableDate {
      timestamp: dt
    }
  }
}

impl Serialize for SerializableDate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("SerializableDate", 1)?;
        s.serialize_field("timestamp", &self.timestamp.timestamp())?;
        s.end()
    }
}

struct DateVisitor;

impl<'de> Visitor<'de> for DateVisitor {
  type Value = SerializableDate;

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    formatter.write_str("a UNIX timestamp")
  }

  fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
  where
    E: de::Error
  {
    Ok(NaiveDateTime::from_timestamp(value, 0).into())
  }

  fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
  where
    E: de::Error
  {
    if (value as i64) < 0 {
      return Err(E::custom(format!("u64 out of range: {}", value)));
    }
    Ok(NaiveDateTime::from_timestamp(value as i64, 0).into())
  }
}

impl<'de> Deserialize<'de> for SerializableDate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
      deserializer.deserialize_i64(DateVisitor)
    }
}
