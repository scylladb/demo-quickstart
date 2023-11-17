use crate::web::routes::*;
use crate::Opt;
use rocket::fs::FileServer;
use rocket::{___internal_relative as relative, routes, Build, Rocket};
use scylla::Session;
use std::sync::Arc;

pub async fn init(session: Arc<Session>, opt: Opt) -> Rocket<Build> {
    rocket::build()
        .mount("/", routes![index, metrics, devices])
        .mount("/", FileServer::from(relative!("public/")))
        .manage(session)
        .manage(opt)
}
