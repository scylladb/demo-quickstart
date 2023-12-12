use crate::db::ddl::DDL;
use anyhow::{anyhow, Result};
use scylla::{Session, SessionBuilder};
use scylla::load_balancing::DefaultPolicy;
use scylla::transport::ExecutionProfile;
use std::env;
use std::time::Duration;
use tokio_retry::{strategy::ExponentialBackoff, Retry};
use tracing::{info, debug};

pub async fn builder(migrate: bool) -> Result<Session> {
    let database_url = env::var("DATABASE_URL")?;

    info!("Connecting to ScyllaDB at {}", database_url);

    let strategy = ExponentialBackoff::from_millis(500).max_delay(Duration::from_secs(20));

    let session = Retry::spawn(strategy, || async {
        let default_policy = DefaultPolicy::builder()
            .prefer_datacenter("datacenter1".to_string())
            .token_aware(true)
            .permit_dc_failover(false)
            .build();


        let profile = ExecutionProfile::builder()
            .load_balancing_policy(default_policy)
            .build();

        let handle = profile.into_handle();

        SessionBuilder::new()
            .known_node(&database_url)
            .default_execution_profile_handle(handle)
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
                debug!("Running Migration {}", query);
                session
                    .query(query, &[])
                    .await
                    .map_err(|e| anyhow!("Error executing migration query: {}", e))?;
            }
        }
    }

    Ok(session)
}
