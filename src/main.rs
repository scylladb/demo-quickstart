mod db;
mod util;
mod web;

use crate::db::connection;
use crate::util::devices;
use anyhow::anyhow;
use std::sync::Arc;
use structopt::StructOpt;
use tokio::{task, try_join};
use util::metrics;
use web::server;
use crate::util::buffer::{flush_buffer_to_db, LatLongBuffer};

#[derive(Debug, Clone, StructOpt)]
pub struct Opt {
    /// read ratio
    #[structopt(default_value = "20")]
    read_ratio: u8,
    /// write ratio
    #[structopt(default_value = "80")]
    write_ratio: u8,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv::dotenv().ok();
    util::logging::init();
    let opt = Opt::from_args();

    if opt.read_ratio + opt.write_ratio != 100 {
        return Err(anyhow!(
            "Invalid ratio configuration. Sum of read_ratio and write_ratio must be 100."
        ));
    }

    let metrics_session = Arc::new(
        connection::builder(true)
            .await
            .expect("Failed to connect to database"),
    );

    let devices_session = Arc::new(
        connection::builder(true)
            .await
            .expect("Failed to connect to database"),
    );

    let web = server::init(metrics_session.clone(), opt.clone()).await;
    tokio::spawn(async { web.launch().await.unwrap() });

    let metrics_task = task::spawn(metrics::worker(
        metrics_session.clone(),
    ));

    let lat_long_buffer = Arc::new(LatLongBuffer::new());
    let devices_task = task::spawn(devices::simulator(
        devices_session.clone(),
        Arc::clone(&lat_long_buffer),
        opt.read_ratio,
        opt.write_ratio,
    ));

    let flush_task = task::spawn(flush_buffer_to_db(
        devices_session.clone(),
        Arc::clone(&lat_long_buffer),
    ));

    // Wait for all tasks to complete
    let (metrics_result, devices_result, flush_result) = try_join!(metrics_task, devices_task, flush_task)?;

    metrics_result?;
    devices_result?;
    flush_result?;

    Ok(())
}
