use crate::commands::config::Target;
use chrono::{DateTime, TimeZone, Utc};

type ResponseCode = u16;

#[derive(Debug, Clone)]
pub struct Entry {
    pub time: DateTime<Utc>,
    pub response_code: ResponseCode,
    pub latency: u128,
    pub target: Target,
}

#[derive(Debug, Clone)]
pub struct EntryDTO {
    pub timestamp_seconds: i64,
    pub response_code: ResponseCode,
    pub latency: u128,
    pub target: Target,
}

impl Entry {
    pub fn new(
        time: DateTime<Utc>,
        latency: u128,
        response_code: ResponseCode,
        target: Target,
    ) -> Entry {
        Entry {
            time,
            response_code,
            latency,
            target,
        }
    }

    pub fn from_dto(dto: EntryDTO) -> Entry {
        Entry {
            time: Utc::timestamp(&Utc, dto.timestamp_seconds, 0),
            response_code: dto.response_code,
            latency: dto.latency,
            target: dto.target,
        }
    }

    pub fn to_dto(&self) -> EntryDTO {
        EntryDTO {
            timestamp_seconds: self.time.timestamp(),
            response_code: self.response_code,
            latency: self.latency,
            target: self.target.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Failure {
    pub time: DateTime<Utc>,
    pub latency: u128,
    pub reason: String,
    pub target: Target,
}

#[derive(Debug, Clone)]
pub struct FailureDTO {
    pub timestamp_seconds: i64,
    pub latency: u128,
    pub reason: String,
    pub target: Target,
}

impl Failure {
    pub fn new(time: DateTime<Utc>, latency: u128, reason: String, target: Target) -> Failure {
        Failure {
            time,
            reason,
            latency,
            target,
        }
    }

    pub fn from_dto(dto: FailureDTO) -> Failure {
        Failure {
            time: Utc::timestamp(&Utc, dto.timestamp_seconds, 0),
            reason: dto.reason,
            latency: dto.latency,
            target: dto.target,
        }
    }

    pub fn to_dto(&self) -> FailureDTO {
        FailureDTO {
            timestamp_seconds: self.time.timestamp(),
            reason: self.reason.clone(),
            latency: self.latency,
            target: self.target.clone(),
        }
    }
}
