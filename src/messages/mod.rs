use chrono::{DateTime, TimeZone, Utc};

#[derive(Debug)]
pub struct Entry {
    pub time: DateTime<Utc>,
    pub response_code: i32,
}

#[derive(Debug)]
pub struct EntryDTO {
    pub timestamp_seconds: i64,
    pub response_code: i32,
}

impl Entry {
    pub fn new(time: DateTime<Utc>, response_code: i32) -> Entry {
        Entry {
            time,
            response_code,
        }
    }

    pub fn from_dto(dto: EntryDTO) -> Entry {
        Entry {
            time: Utc::timestamp(&Utc, dto.timestamp_seconds, 0),
            response_code: dto.response_code,
        }
    }

    pub fn to_dto(&self) -> EntryDTO {
        EntryDTO {
            timestamp_seconds: self.time.timestamp(),
            response_code: self.response_code,
        }
    }
}
