use fake::faker::internet::raw::*;
use fake::locales::EN;
use fake::Fake;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use scylla::prepared_statement::PreparedStatement;
use scylla::transport::session::Session;
use scylla::IntoTypedRows;
use std::sync::Arc;
use tracing::{debug, error};
use uuid::Uuid;
use crate::util::buffer::{LatLong, LatLongBuffer};
use crate::util::coords;
use crate::util::geo::get_geo_hash;

const READ_DEVICE: &str = "SELECT geo_hash, device_id FROM demo.devices WHERE geo_hash = ? LIMIT 1";
const INSERT_DEVICE: &str = "INSERT INTO devices (device_id, geo_hash, lat, lng, ipv4) VALUES (?, ?, ?, ?, ?)";

pub async fn simulator(
    session: Arc<Session>,
    lat_long_buffer: Arc<LatLongBuffer>,
    read_ratio: u8,
    write_ratio: u8,
) -> Result<(), anyhow::Error> {
    let total_ratio = read_ratio + write_ratio;
    let mut rng = StdRng::from_entropy();

    let read_device: PreparedStatement = session.prepare(
        READ_DEVICE)
        .await?;

    let insert_device: PreparedStatement = session.prepare(
        INSERT_DEVICE)
        .await?;

    loop {
        let rand_num: u8 = rng.gen_range(0..total_ratio);

        let idx = rng.gen_range(0..=coords::LATLONGS.len() - 1);
        let (lat_long, geo_hash) = get_geo_hash(idx);

        if rand_num < read_ratio {
            // Simulate a read operation
            match session
                .execute(&read_device,
                    (&geo_hash.clone(),)
                )
                .await
            {
                Ok(response) => {
                    let rows = response.rows.unwrap_or_default();
                    for row in rows.into_typed::<(String, Uuid)>() {
                        match row {
                            Ok((geo_hash, device_id)) => debug!("Geo Hash: {} for device ID: {}", geo_hash, device_id),
                            Err(e) => error!("Failed to parse row: {}", e),
                        }
                    }
                }
                Err(e) => error!("Failed to perform read operation: {}", e),
            }
        } else {
            // Simulate a write operation
            let ipv4: String = IPv4(EN).fake();

            let device_id = Uuid::new_v4();

            match session.execute(&insert_device,
                (device_id, geo_hash.clone(), lat_long.0, lat_long.1, ipv4)
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
