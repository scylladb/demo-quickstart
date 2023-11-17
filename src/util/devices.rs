use fake::faker::internet::raw::*;
use fake::locales::EN;
use fake::Fake;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use scylla::batch::Batch;
use scylla::transport::session::Session;
use scylla::IntoTypedRows;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error};
use uuid::Uuid;
use crate::util::coords;


pub async fn simulator(
    session: Arc<Session>,
    read_ratio: u8,
    write_ratio: u8,
    ops_per_iter: u8,
) -> Result<(), anyhow::Error> {
    let total_ratio = read_ratio + write_ratio;
    let mut rng = StdRng::from_entropy();
    let known_uuid = Uuid::from_str("dce006bf-470c-4ea6-93e2-209f7d520460")?;
    loop {
        let mut batch: Batch = Default::default();
        let mut batch_values = Vec::with_capacity(100);
        let timestamp_now = chrono::Utc::now().timestamp_millis();
        let timestamp_ago = timestamp_now - 3 * 1000;
        for (index, _value) in (0..ops_per_iter).enumerate() {
            let rand_num: u8 = rng.gen_range(0..total_ratio);
            if rand_num < read_ratio {
                // Simulate a read operation.
                match session
                    .query(
                        "SELECT sensor_data FROM devices WHERE device_id = ? AND timestamp > ? LIMIT 1",
                        (known_uuid, timestamp_ago),
                    )
                    .await
                {
                    Ok(response) => {
                        let rows = response.rows.unwrap_or_default();
                        for row in rows.into_typed::<(i64,)>() {
                            // Parse row as float
                            match row {
                                Ok((sensor_data,)) => debug!("Sensor data: {}", sensor_data),
                                Err(e) => error!("Failed to parse row: {}", e),
                            }
                        }
                    }
                    Err(e) => error!("Failed to perform read operation: {}", e),
                }
            } else {
                // Simulate a write operation.
                let uuid = if index == 0 {
                    known_uuid
                } else {
                    Uuid::new_v4()
                };
                let sensor_data: i64 = rng.gen_range(1..=25); // Generate random sensor data.
                let ipv4: String = IPv4(EN).fake();

                let idx = rng.gen_range(0..=5_000);
                let lat_long = coords::LATLONGS[idx];

                batch.append_statement(
                    "INSERT INTO devices (device_id, timestamp, sensor_data, lat, lng, ipv4) VALUES (?, ?, ?, ?, ?, ?)",
                );

                batch_values.push((
                    uuid,
                    chrono::Utc::now().timestamp_millis(),
                    sensor_data,
                    lat_long.0,
                    lat_long.1,
                    ipv4,
                ));
            }
        }
        let prepared_batch: Batch = session.prepare_batch(&batch).await?;
        session.batch(&prepared_batch, batch_values).await?;
        debug!("Batch written successfully!");

        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}
