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

#[derive(Debug, Clone, StructOpt)]
pub struct Opt {
    /// read ratio
    #[structopt(default_value = "20")]
    read_ratio: u8,
    /// write ratio
    #[structopt(default_value = "80")]
    write_ratio: u8,
    /// operations per iteration
    #[structopt(default_value = "50")]
    ops_per_iter: u8,
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
        devices_session.clone(),
    ));
    let devices_task = task::spawn(devices::simulator(
        devices_session.clone(),
        opt.read_ratio,
        opt.write_ratio,
        opt.ops_per_iter,
    ));
    let (metrics_result, devices_result) = try_join!(metrics_task, devices_task)?;

    metrics_result?;
    devices_result?;

    Ok(())
}
