use rocket::serde::Serialize;
use scylla::FromRow;

#[derive(Clone, Debug, Serialize, FromRow)]
pub struct Metric {
    pub node_id: String,
    pub timestamp: i64,
    pub latency_read_max: i64,
    pub latency_write_max: i64,
    pub reads_total: i64,
    pub writes_total: i64,
}
