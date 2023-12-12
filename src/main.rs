mod db;
mod util;
mod web;

use crate::db::connection;
use crate::util::devices;
use anyhow::anyhow;
use std::sync::Arc;
use std::process;
use futures::stream::FuturesUnordered;
use futures::TryStreamExt;
use structopt::StructOpt;
use tokio::{signal, task};
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
    /// simulators
    #[structopt(default_value = "50")]
    simulators: u8,
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

    let db_session = Arc::new(
        connection::builder(true)
            .await
            .expect("Failed to connect to database"),
    );

    let ctrl_c_handler = tokio::spawn(async {
        signal::ctrl_c().await.expect("Failed to listen for Ctrl-C");
        process::exit(0);
    });

    let web = server::init(db_session.clone(), opt.clone()).await;
    tokio::spawn(async { web.launch().await.unwrap() });

    let metrics_task = task::spawn(metrics::worker(
        db_session.clone(),
    ));

    let lat_long_buffer = Arc::new(LatLongBuffer::new());
    let flush_task = task::spawn(flush_buffer_to_db(
        db_session.clone(),
        Arc::clone(&lat_long_buffer),
    ));

    let devices_tasks: FuturesUnordered<_> = (0..opt.simulators).map(|_| {
        task::spawn(devices::simulator(
            db_session.clone(),
            Arc::clone(&lat_long_buffer),
            opt.read_ratio,
            opt.write_ratio,
        ))
    }).collect();
    let devices_result = devices_tasks.try_collect::<Vec<_>>().await?;

    let (metrics_result, flush_result) = tokio::try_join!(metrics_task, flush_task)?;
    metrics_result?;
    flush_result?;

    devices_result.into_iter().collect::<Result<Vec<_>, _>>()?;

    tokio::select! {
        _ = ctrl_c_handler => (),
    }

    Ok(())
}
