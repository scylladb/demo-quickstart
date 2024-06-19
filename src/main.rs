use std::process;
use std::sync::Arc;

use anyhow::anyhow;
use futures::stream::FuturesUnordered;
use futures::TryStreamExt;
use structopt::StructOpt;
use tokio::{signal, task};

use util::metrics;
use web::server;

use crate::db::connection;
use crate::util::devices;

mod db;
mod util;
mod web;

#[derive(Debug, Clone, StructOpt)]
pub struct Opt {
    /// read ratio
    #[structopt(default_value = "80")]
    read_ratio: u8,
    /// write ratio
    #[structopt(default_value = "20")]
    write_ratio: u8,
    /// simulators
    #[structopt(default_value = "30")]
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

    let devices_tasks: FuturesUnordered<_> = (0..opt.simulators).map(|_| {
        task::spawn(devices::simulator(
            db_session.clone(),
            opt.read_ratio,
            opt.write_ratio,
        ))
    }).collect();
    let devices_result = devices_tasks.try_collect::<Vec<_>>().await?;

    let (metrics_result, ) = tokio::try_join!(metrics_task)?;
    metrics_result?;

    devices_result.into_iter().collect::<Result<Vec<_>, _>>()?;

    tokio::select! {
        _ = ctrl_c_handler => (),
    }

    Ok(())
}
