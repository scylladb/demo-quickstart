use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Result};
use regex::Regex;
use scylla::Session;
use tracing::{debug, error};

use crate::db::models;

pub async fn writer(
    metrics_session: Arc<Session>,
) -> Result<models::Metric, anyhow::Error> {
    let metrics_url = env::var("METRICS_URL")?;
    let endpoint = format!("http://{}/metrics", metrics_url);

    let latencies = fetch_max_latency_metrics(endpoint.as_str()).await?;
    let reads = fetch_total_read_metrics(endpoint.as_str()).await?;
    let writes = fetch_total_write_metrics(endpoint.as_str()).await?;

    let metric = models::Metric {
        node_id: "ABCD".parse().unwrap(),
        timestamp: chrono::Utc::now().timestamp_millis(),
        reads_total: reads as i64,
        writes_total: writes as i64,
        latency_read_max: *latencies.get("read").unwrap_or(&0) as i64,
        latency_write_max: *latencies.get("write").unwrap_or(&0) as i64,
    };

    let cql = metrics_session
        .prepare(
            "INSERT INTO demo.metrics \
            (node_id, timestamp, reads_total, writes_total, latency_read_max, latency_write_max) \
            VALUES (?, ?, ?, ?, ?, ?)",
        )
        .await?;

    match metrics_session
        .execute(
            &cql,
            (
                metric.node_id.clone(),
                metric.timestamp,
                metric.reads_total,
                metric.writes_total,
                metric.latency_read_max,
                metric.latency_write_max,
            ),
        )
        .await
    {
        Ok(_) => Ok(metric),
        Err(e) => {
            error!("Failed to execute metrics write: {:?}", e);
            Err(anyhow!("Failed to execute metrics write: {:?}", e))
        }
    }
}

async fn fetch_max_latency_metrics(endpoint: &str) -> Result<HashMap<String, i64>> {
    let client = reqwest::Client::new();
    let response = client.get(endpoint).send().await?.text().await?;

    let re = Regex::new("scylla_storage_proxy_coordinator_(\\w+)_latency_summary\\{quantile=\"0\\.990000\",.*,shard=\"(\\d+)\"\\} (\\d+)").unwrap();

    let mut max_latencies = HashMap::new();

    for line in response.lines() {
        if let Some(caps) = re.captures(line) {
            let operation = caps[1].to_string();
            let latency: i64 = caps[3].parse().unwrap_or(0);

            max_latencies.entry(operation)
                .and_modify(|e| *e = i64::max(*e, latency))
                .or_insert(latency);
        }
    }

    if max_latencies.is_empty() {
        return Err(anyhow!("No latency data found"));
    }

    Ok(max_latencies)
}

async fn fetch_total_read_metrics(endpoint: &str) -> Result<i64> {
    let client = reqwest::Client::new();
    let response = client.get(endpoint).send().await?.text().await?;

    let re_total = Regex::new("scylla_cql_reads\\{shard=\"(\\d+)\"\\} (\\d+)").unwrap();
    let re_internal = Regex::new("scylla_cql_reads_per_ks\\{ks=\"system\", shard=\"(\\d+)\", who=\"internal\"\\} (\\d+)").unwrap();

    let mut total_reads = 0;
    let mut internal_reads = 0;

    for line in response.lines() {
        if let Some(caps) = re_total.captures(line) {
            total_reads += caps[2].parse::<i64>().unwrap_or(0);
        } else if let Some(caps) = re_internal.captures(line) {
            internal_reads += caps[2].parse::<i64>().unwrap_or(0);
        }
    }

    let net_reads = total_reads - internal_reads;

    Ok(net_reads)
}

async fn fetch_total_write_metrics(endpoint: &str) -> Result<i64> {
    let client = reqwest::Client::new();
    let response = client.get(endpoint).send().await?.text().await?;

    let re_total = Regex::new("scylla_cql_inserts\\{conditional.+?shard=\"(\\d+)\"\\} (\\d+)").unwrap();
    let re_internal = Regex::new("scylla_cql_inserts_per_ks\\{conditional.+?ks=\"system\", shard=\"(\\d+)\", who=\"internal\"\\} (\\d+)").unwrap();

    let mut total_writes = 0;
    let mut internal_writes = 0;

    for line in response.lines() {
        if let Some(caps) = re_total.captures(line) {
            total_writes += caps[2].parse::<i64>().unwrap_or(0);
        } else if let Some(caps) = re_internal.captures(line) {
            internal_writes += caps[2].parse::<i64>().unwrap_or(0);
        }
    }

    let net_reads = total_writes - internal_writes;

    Ok(net_reads)
}

pub async fn worker(
    metrics_session: Arc<Session>,
) -> Result<(), anyhow::Error> {
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    loop {
        interval.tick().await;
        match writer(metrics_session.clone()).await {
            Ok(_) => debug!("Metrics written successfully"),
            Err(e) => error!("Error writing metrics: {}", e),
        }
    }
}
