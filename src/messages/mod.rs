use crate::commands::config::Target;
use chrono::{DateTime, TimeZone, Utc};

#[derive(Debug)]
pub struct Entry {
    pub time: DateTime<Utc>,
    pub response_code: i32,
    pub target: Target,
}

#[derive(Debug)]
pub struct EntryDTO {
    pub timestamp_seconds: i64,
    pub response_code: i32,
    pub target: Target,
}

impl Entry {
    pub fn new(time: DateTime<Utc>, response_code: i32, target: Target) -> Entry {
        Entry {
            time,
            response_code,
            target,
        }
    }

    pub fn from_dto(dto: EntryDTO) -> Entry {
        Entry {
            time: Utc::timestamp(&Utc, dto.timestamp_seconds, 0),
            response_code: dto.response_code,
            target: dto.target,
        }
    }

    pub fn to_dto(&self) -> EntryDTO {
        EntryDTO {
            timestamp_seconds: self.time.timestamp(),
            response_code: self.response_code,
            target: self.target.clone(),
        }
    }
}
