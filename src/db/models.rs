use rocket::serde::Serialize;
use scylla::FromRow;

#[derive(Clone, Debug, Serialize, FromRow)]
pub struct Metric {
    pub node_id: String,
    pub timestamp: i64,
    pub errors_iter_num: i64,
    pub errors_num: i64,
    pub latency_avg_ms: i64,
    pub latency_percentile_ms: i64,
    pub queries_iter_num: i64,
    pub queries_num: i64,
}
