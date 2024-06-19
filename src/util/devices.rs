use std::sync::Arc;

use fake::Fake;
use fake::faker::internet::raw::*;
use fake::locales::EN;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use scylla::IntoTypedRows;
use scylla::prepared_statement::PreparedStatement;
use scylla::transport::session::Session;
use tracing::{debug, error};
use uuid::Uuid;

use crate::util::coords;
use crate::util::geo::get_geo_hash;

const READ_DEVICE: &str = "SELECT device_id, geo_hash FROM demo.devices WHERE device_id = ? LIMIT 1";
const INSERT_DEVICE: &str = "INSERT INTO devices (device_id, geo_hash, lat, lng, ipv4) VALUES (?, ?, ?, ?, ?)";

pub async fn simulator(
    session: Arc<Session>,
    read_ratio: u8,
    write_ratio: u8,
) -> Result<(), anyhow::Error> {
    let total_ratio = read_ratio + write_ratio;
    let mut rng = StdRng::from_entropy();
    let uniques = 3200;

    let read_device: PreparedStatement = session.prepare(
        READ_DEVICE)
        .await?;

    let insert_device: PreparedStatement = session.prepare(
        INSERT_DEVICE)
        .await?;

    let mut uuids = Vec::new();
    for _ in 0..uniques {
        uuids.push(Uuid::new_v4());
    }

    let mut ipv4s: Vec<String> = Vec::new();
    for _ in 0..uniques {
        ipv4s.push(IPv4(EN).fake());
    }

    let mut geo_hashes: Vec<((f64, f64), String)> = Vec::new();
    for _ in 0..=coords::LATLONGS.len() - 1 {
        let idx = rng.gen_range(0..=coords::LATLONGS.len() - 1);
        geo_hashes.push(get_geo_hash(idx));
    }

    loop {
        let rand_num: u8 = rng.gen_range(0..total_ratio);
        let unique = rng.gen_range(0..uniques);

        let (lat_long, geo_hash) = geo_hashes[unique].clone();
        let device_id = uuids[unique];
        let ipv4: &String = &ipv4s[unique];

        if rand_num < read_ratio {
            // Simulate a read operation
            match session
                .execute(&read_device,
                         (&device_id.clone(),),
                )
                .await
            {
                Ok(response) => {
                    let rows = response.rows.unwrap_or_default();
                    for row in rows.into_typed::<(Uuid, String)>() {
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
            match session.execute(&insert_device,
                                  (device_id, geo_hash.clone(), lat_long.0, lat_long.1, ipv4),
            )
                .await
            {
                Ok(_) => debug!("Write operation successful"),
                Err(e) => error!("Failed to perform write operation: {}", e),
            }
        }
    }
}
