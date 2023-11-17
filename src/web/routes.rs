use crate::db::models::*;
use crate::Opt;
use rocket::fs::NamedFile;
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::serde::Serialize;
use rocket::{get, State};
use scylla::query::Query;
use scylla::{FromRow, IntoTypedRows, Session};
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

#[get("/")]
pub async fn index() -> Option<NamedFile> {
    NamedFile::open(Path::new("public/index.html")).await.ok()
}

#[get("/metrics", rank = 1)]
pub async fn metrics(
    session: &State<Arc<Session>>,
    opt: &State<Opt>,
) -> Result<Json<Vec<RateMetric>>, status::Custom<String>> {
    let timestamp_now = chrono::Utc::now().timestamp_millis();
    let timestamp_minute_ago = timestamp_now - 60 * 1000;

    let cql_query = Query::new("SELECT * FROM metrics WHERE timestamp > ? AND timestamp <= ?;");

    let rows = session
        .query(cql_query, (timestamp_minute_ago, timestamp_now))
        .await
        .map_err(|err| status::Custom(Status::InternalServerError, err.to_string()))?
        .rows
        .unwrap_or_default();

    let metrics: Vec<Metric> = rows.into_typed().filter_map(Result::ok).collect();

    let mut rate_metrics: Vec<RateMetric> = Vec::new();

    for windows in metrics.windows(2) {
        if let [prev, curr] = windows {
            if curr.timestamp == prev.timestamp {
                continue;
            }

            let ops_per_second = (curr.queries_num - prev.queries_num) as f64
                / ((curr.timestamp - prev.timestamp) as f64 / 1000.0) // milliseconds to seconds
                * 100.0; // loops per iteration

            let rate_metric = RateMetric {
                node_id: curr.node_id.clone(),
                timestamp: curr.timestamp,
                ops_per_second,
                reads_per_second: ops_per_second * (opt.read_ratio as f64 / 100.0),
                writes_per_second: ops_per_second * (opt.write_ratio as f64 / 100.0),
                errors_per_second: (curr.errors_num - prev.errors_num) as f64
                    / ((curr.timestamp - prev.timestamp) as f64 / 1000.0)
                    * 100.0,
                latency_mean_ms: curr.latency_avg_ms,
                latency_p99_ms: curr.latency_percentile_ms,
                total_reads: curr.queries_num as f64 * (opt.read_ratio as f64 / 100.0),
                total_writes: curr.queries_num as f64 * (opt.write_ratio as f64 / 100.0),
                total_queries_iter: curr.queries_iter_num,
                total_errors: curr.errors_num,
                total_errors_iter: curr.errors_iter_num,
            };
            rate_metrics.push(rate_metric);
        }
    }

    // rate_metrics.reverse();

    Ok(Json(rate_metrics))
}

#[derive(Serialize)]
pub struct RateMetric {
    pub node_id: String,
    pub timestamp: i64,
    pub ops_per_second: f64,
    pub reads_per_second: f64,
    pub writes_per_second: f64,
    pub errors_per_second: f64,
    pub latency_mean_ms: i64,
    pub latency_p99_ms: i64,
    pub total_reads: f64,
    pub total_writes: f64,
    pub total_queries_iter: i64,
    pub total_errors: i64,
    pub total_errors_iter: i64,
}

#[get("/devices", rank = 2)]
pub async fn devices(
    session: &State<Arc<Session>>,
) -> Result<Json<Vec<Device>>, status::Custom<String>> {
    let timestamp_now = chrono::Utc::now().timestamp_millis();
    let timestamp_minute_ago = timestamp_now - 60 * 1000;

    let cql_query =
        Query::new("SELECT * FROM devices WHERE timestamp > ? AND timestamp <= ? LIMIT 1000;");

    let rows = session
        .query(cql_query, (timestamp_minute_ago, timestamp_now))
        .await
        .map_err(|err| status::Custom(Status::InternalServerError, err.to_string()))?
        .rows
        .unwrap_or_default();

    let devices: Vec<Device> = rows.into_typed().filter_map(Result::ok).collect();

    Ok(Json(devices))
}

#[derive(Clone, Debug, FromRow, Serialize)]
pub struct Device {
    pub uuid: Uuid,
    pub timestamp: i64,
    pub ipv4: String,
    pub lat: f64,
    pub lng: f64,
    pub sensor_data: i64,
}
