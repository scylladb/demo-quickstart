use fake::faker::internet::raw::*;
use fake::locales::EN;
use fake::Fake;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use scylla::transport::session::Session;
use scylla::IntoTypedRows;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error};
use uuid::Uuid;
use crate::util::buffer::{LatLong, LatLongBuffer};
use crate::util::coords;
use crate::util::geo::get_geo_hash;

pub async fn simulator(
    session: Arc<Session>,
    lat_long_buffer: Arc<LatLongBuffer>,
    read_ratio: u8,
    write_ratio: u8,
) -> Result<(), anyhow::Error> {
    let total_ratio = read_ratio + write_ratio;
    let mut rng = StdRng::from_entropy();

    loop {
        let uuid = Uuid::new_v4();
        let rand_num: u8 = rng.gen_range(0..total_ratio);

        let idx = rng.gen_range(0..=coords::LATLONGS.len() - 1);
        let (lat_long, geo_hash) = get_geo_hash(idx);

        if rand_num < read_ratio {
            // Simulate a read operation
            match session
                .query(
                    "SELECT geo_hash, device_id FROM devices WHERE geo_hash = ? AND device_id = ? LIMIT 1",
                    (geo_hash.clone(), uuid),
                )
                .await
            {
                Ok(response) => {
                    let rows = response.rows.unwrap_or_default();
                    for row in rows.into_typed::<(i64,)>() {
                        match row {
                            Ok((sensor_data,)) => debug!("Sensor data: {}", sensor_data),
                            Err(e) => error!("Failed to parse row: {}", e),
                        }
                    }
                }
                Err(e) => error!("Failed to perform read operation: {}", e),
            }
        } else {
            // Simulate a write operation
            let ipv4: String = IPv4(EN).fake();

            let new_device_id = Uuid::new_v4();

            match session
                .query(
                    "INSERT INTO devices (device_id, geo_hash, lat, lng, ipv4) VALUES (?, ?, ?, ?, ?)",
                    (new_device_id, geo_hash.clone(), lat_long.0, lat_long.1, ipv4)
                )
                .await
            {
                Ok(_) => debug!("Write operation successful"),
                Err(e) => error!("Failed to perform write operation: {}", e),
            }

            lat_long_buffer.add(LatLong { lat: lat_long.0, lng: lat_long.1 });
        }
    }
}
