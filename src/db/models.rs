use rocket::serde::Serialize;
use scylla::frame::value::CqlTimestamp;
use scylla::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, FromRow)]
pub struct Metric {
    pub node_id: String,
    pub timestamp: CqlTimestamp,
    pub latency_read_max: i64,
    pub latency_write_max: i64,
    pub reads_total: i64,
    pub writes_total: i64,
}

#[derive(Debug, Serialize)]
pub struct RateMetric {
    pub node_id: String,
    pub timestamp: i64,
    pub reads_per_second: f64,
    pub writes_per_second: f64,
    pub ops_per_second: f64,
    pub reads_total: i64,
    pub writes_total: i64,
    pub latency_read_max: f64,
    pub latency_write_max: f64,
}

#[derive(Clone, Debug, FromRow, Serialize)]
pub struct Device {
    pub device_id: Uuid,
    pub geo_hash: String,
    pub ipv4: String,
    pub lat: f64,
    pub lng: f64,
}
