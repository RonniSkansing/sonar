use crate::commands::config::Target;
use chrono::{DateTime, TimeZone, Utc};

type ResponseCode = u16;

#[derive(Debug)]
pub struct Entry {
    pub time: DateTime<Utc>,
    pub response_code: ResponseCode,
    pub target: Target,
}

#[derive(Debug)]
pub struct EntryDTO {
    pub timestamp_seconds: i64,
    pub response_code: ResponseCode,
    pub target: Target,
}

impl Entry {
    pub fn new(time: DateTime<Utc>, response_code: ResponseCode, target: Target) -> Entry {
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

#[derive(Debug)]
pub struct Failure {
    pub time: DateTime<Utc>,
    pub reason: String,
    pub target: Target,
}

#[derive(Debug)]
pub struct FailureDTO {
    pub timestamp_seconds: i64,
    pub reason: String,
    pub target: Target,
}

impl Failure {
    pub fn new(time: DateTime<Utc>, reason: String, target: Target) -> Failure {
        Failure {
            time,
            reason,
            target,
        }
    }

    pub fn from_dto(dto: FailureDTO) -> Failure {
        Failure {
            time: Utc::timestamp(&Utc, dto.timestamp_seconds, 0),
            reason: dto.reason,
            target: dto.target,
        }
    }

    pub fn to_dto(&self) -> FailureDTO {
        FailureDTO {
            timestamp_seconds: self.time.timestamp(),
            reason: self.reason.clone(),
            target: self.target.clone(),
        }
    }
}
