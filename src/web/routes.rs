use std::path::Path;
use std::sync::Arc;

use rocket::{get, State};
use rocket::fs::NamedFile;
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use scylla::{IntoTypedRows, Session};
use scylla::frame::value::CqlTimestamp;
use scylla::query::Query;

use crate::db::models::*;

#[get("/")]
pub async fn index() -> Option<NamedFile> {
    NamedFile::open(Path::new("public/index.html")).await.ok()
}

#[get("/metrics", rank = 1)]
pub async fn metrics(
    session: &State<Arc<Session>>,
) -> Result<Json<Vec<RateMetric>>, status::Custom<String>> {
    let timestamp_now = chrono::Utc::now().timestamp_millis();
    let timestamp_minute_ago = timestamp_now - 60 * 1000;

    let cql_query = Query::new("SELECT * FROM metrics WHERE node_id = ? AND timestamp > ? AND timestamp <= ?;");

    let rows = session
        .query(cql_query, ("ABCD", &CqlTimestamp(timestamp_minute_ago), &CqlTimestamp(timestamp_now)))
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

            let reads_per_second: f64 = ((curr.reads_total - prev.reads_total) as f64
                / ((curr.timestamp.0 - prev.timestamp.0) as f64 / 1000.0)).round();

            let writes_per_second = ((curr.writes_total - prev.writes_total) as f64
                / ((curr.timestamp.0 - prev.timestamp.0) as f64 / 1000.0)).round();

            let ops_per_second = reads_per_second + writes_per_second;

            let rate_metric = RateMetric {
                node_id: curr.node_id.clone(),
                timestamp: curr.timestamp.0,
                reads_per_second,
                writes_per_second,
                ops_per_second,
                reads_total: curr.reads_total,
                writes_total: curr.writes_total,
                latency_read_max: curr.latency_read_max as f64 / 1000.0,
                latency_write_max: curr.latency_write_max as f64 / 1000.0,
            };
            rate_metrics.push(rate_metric);
        }
    }

    Ok(Json(rate_metrics))
}

#[get("/devices", rank = 2)]
pub async fn devices(
    session: &State<Arc<Session>>,
) -> Result<Json<Vec<Device>>, status::Custom<String>> {
    let cql_query =
        Query::new("SELECT * FROM devices LIMIT 1024;");

    let rows = session
        .query(cql_query, ())
        .await
        .map_err(|err| status::Custom(Status::InternalServerError, err.to_string()))?
        .rows
        .unwrap_or_default();

    let devices: Vec<Device> = rows.into_typed().filter_map(Result::ok).collect();

    Ok(Json(devices))
}
