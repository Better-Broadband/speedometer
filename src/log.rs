use crate::Error;
use chrono::prelude::*;
use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, PartialEq)]
pub enum Bandwidth {
    Bits(f64),
    Kilobits(f64),
    Megabits(f64),
    Gigabits(f64),
    Terabits(f64),
}

impl Serialize for Bandwidth {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value_in_bits = match self {
            Bandwidth::Bits(n) => *n,
            Bandwidth::Kilobits(n) => *n * 1024.0,
            Bandwidth::Megabits(n) => *n * 1024.0f64.powi(2),
            Bandwidth::Gigabits(n) => *n * 1024.0f64.powi(3),
            Bandwidth::Terabits(n) => *n * 1024.0f64.powi(4),
        };
        serializer.serialize_f64(value_in_bits)
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub struct LogRecord {
    device_name: String,
    timestamp: NaiveDateTime,
    test_name: String,
    download: Bandwidth,
    upload: Bandwidth,
    ping: f32,
    client_ip: String,
    client_lat: Option<String>,
    client_lon: Option<String>,
}

impl LogRecord {
    pub fn from_json(bytes: &[u8]) -> Result<Self, Box<Error>> {
        let raw: RawLogRecord = serde_json::from_slice(bytes).map_err(Error::JsonParseError)?;
        let log: LogRecord = raw.try_into()?;
        Ok(log)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub struct RawLogRecord {
    #[serde(rename(deserialize = "MurakamiLocation"))]
    device_name: String,
    #[serde(rename(deserialize = "TestStartTime"))]
    timestamp: NaiveDateTime,
    test_name: String,
    download_value: f64,
    download_unit: String,
    upload_value: f64,
    upload_unit: String,
    #[serde(alias = "MinRTTValue", alias = "Ping")]
    ping: f32,
    #[serde(alias = "ClientIP")]
    client_ip: String,
    client_lon: Option<String>,
    client_lat: Option<String>,
}

impl TryInto<LogRecord> for RawLogRecord {
    type Error = Error;

    fn try_into(self) -> Result<LogRecord, Self::Error> {
        let download = match self.download_unit.as_str() {
            "Bit/s" => Bandwidth::Bits(self.download_value),
            "Kbit/s" => Bandwidth::Bits(self.download_value),
            "Mbit/s" => Bandwidth::Megabits(self.download_value),
            "Gbit/s" => Bandwidth::Gigabits(self.download_value),
            "Tbit/s" => Bandwidth::Terabits(self.download_value),
            _ => return Err(Error::ConvertRawLogError(self)),
        };
        let upload = match self.upload_unit.as_str() {
            "Bit/s" => Bandwidth::Bits(self.upload_value),
            "Kbit/s" => Bandwidth::Bits(self.upload_value),
            "Mbit/s" => Bandwidth::Megabits(self.upload_value),
            "Gbit/s" => Bandwidth::Gigabits(self.upload_value),
            "Tbit/s" => Bandwidth::Terabits(self.upload_value),
            _ => return Err(Error::ConvertRawLogError(self)),
        };

        Ok(LogRecord {
            upload,
            download,
            device_name: self.device_name,
            timestamp: self.timestamp,
            test_name: self.test_name,
            ping: self.ping,
            client_ip: self.client_ip,
            client_lon: self.client_lon,
            client_lat: self.client_lat,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{LogRecord, RawLogRecord};

    const NDT5_LOG: &[u8] = include_bytes!("../test_data/ndt5-devicename-etc.jsonl");
    const NDT7_LOG: &[u8] = include_bytes!("../test_data/ndt7-devicename2-etc.jsonl");
    const SINGLE_STREAM_LOG: &[u8] =
        include_bytes!("../test_data/speedtest-cli-single-stream-devicename3-etc.jsonl");
    const MULTI_STREAM_LOG: &[u8] =
        include_bytes!("../test_data/speedtest-cli-multi-stream-devicename4-etc.jsonl");

    #[test]
    fn parse_logfile_ndt5() {
        let log: RawLogRecord = serde_json::from_slice(NDT5_LOG).expect("raw log must parse");
        let log: LogRecord = log.try_into().expect("log must convert successfully");
        assert_eq!(log.test_name, "ndt5");
    }

    #[test]
    fn parse_logfile_ndt7() {
        let log: RawLogRecord = serde_json::from_slice(NDT7_LOG).expect("raw log must parse");
        let log: LogRecord = log.try_into().expect("log must convert successfully");
        assert_eq!(log.test_name, "ndt7");
    }
    #[test]
    fn parse_logfile_single_stream() {
        let log: RawLogRecord =
            serde_json::from_slice(SINGLE_STREAM_LOG).expect("raw log must parse");
        let log: LogRecord = log.try_into().expect("log must convert successfully");
        assert_eq!(log.test_name, "speedtest-cli-single-stream");
    }
    #[test]
    fn parse_logfile_multi_stream() {
        let log: RawLogRecord =
            serde_json::from_slice(MULTI_STREAM_LOG).expect("raw log must parse");
        let log: LogRecord = log.try_into().expect("log must convert successfully");
        assert_eq!(log.test_name, "speedtest-cli-multi-stream");
    }
}
