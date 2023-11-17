use crate::db::ddl::DDL;
use anyhow::{anyhow, Result};
use scylla::{Session, SessionBuilder};
use std::env;
use std::time::Duration;
use tokio_retry::{strategy::ExponentialBackoff, Retry};

use tracing::info;

pub async fn builder(migrate: bool) -> Result<Session> {
    let database_url = env::var("DATABASE_URL")?;

    info!("Connecting to ScyllaDB at {}", database_url);

    let strategy = ExponentialBackoff::from_millis(500).max_delay(Duration::from_secs(20));

    let session = Retry::spawn(strategy, || async {
        SessionBuilder::new()
            .known_node(&database_url)
            .build()
            .await
    })
    .await
    .map_err(|e| anyhow!("Error connecting to the database: {}", e))?;

    if migrate {
        let schema_query = DDL.trim().replace('\n', " ");
        for q in schema_query.split(';') {
            let query = q.to_owned() + ";";
            if !query.starts_with("--") && query.len() > 1 {
                info!("Running Migration {}", query);
                session
                    .query(query, &[])
                    .await
                    .map_err(|e| anyhow!("Error executing migration query: {}", e))?;
            }
        }
    }

    Ok(session)
}
