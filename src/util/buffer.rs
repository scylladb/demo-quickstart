use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use scylla::Session;
use scylla::prepared_statement::PreparedStatement;
use std::time::Duration;
use tracing::{error};
use anyhow::Error;

const INSERT_LATLONG: &str = "INSERT INTO unique_lat_lng (lat, lng) VALUES (?, ?)";

#[derive(Clone, Debug)]
pub struct LatLong {
    pub(crate) lat: f64,
    pub(crate) lng: f64,
}

impl Hash for LatLong {
    fn hash<H: Hasher>(&self, state: &mut H) {
        format!("{:.6},{:.6}", self.lat, self.lng).hash(state);
    }
}
impl PartialEq for LatLong {
    fn eq(&self, other: &Self) -> bool {
        // Define a threshold for how close the floats need to be to be considered equal
        const THRESHOLD: f64 = 0.000001; // Adjust as needed

        (self.lat - other.lat).abs() < THRESHOLD && (self.lng - other.lng).abs() < THRESHOLD
    }
}
impl Eq for LatLong {}

pub struct LatLongBuffer {
    buffer: Mutex<HashSet<LatLong>>,
}

impl LatLongBuffer {
    pub(crate) fn new() -> Self {
        LatLongBuffer {
            buffer: Mutex::new(HashSet::new()),
        }
    }

    pub(crate) fn add(&self, lat_long: LatLong) {
        let mut buffer = self.buffer.lock().unwrap();
        buffer.insert(lat_long);
    }

    fn flush(&self) -> HashSet<LatLong> {
        let mut buffer = self.buffer.lock().unwrap();
        std::mem::take(&mut *buffer)
    }
}

pub async fn flush_buffer_to_db(
    session: Arc<Session>,
    lat_long_buffer: Arc<LatLongBuffer>,
) -> Result<(), Error> {
    let mut interval = tokio::time::interval(Duration::from_secs(3));

    let insert_statement: PreparedStatement = session
        .prepare(INSERT_LATLONG)
        .await?;

    loop {
        interval.tick().await;
        let unique_pairs = lat_long_buffer.flush();

        for lat_long in unique_pairs {
            // Perform database insert for each unique lat-long pair
            if let Err(e) = session
                .execute(
                    &insert_statement,
                    (lat_long.lat, lat_long.lng)
                )
                .await
            {
                error!("Failed to insert lat-long to unique_lat_lng: {}", e);
            }
        }
    }
}
