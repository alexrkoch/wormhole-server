mod config;
use actix_web::{get, App, HttpServer, Responder};
use anyhow::Result as AnyhowResult;

#[get("/")]
#[tracing::instrument]
async fn root() -> impl Responder {
    "I am root"
}

#[tokio::main]
async fn main() -> AnyhowResult<()> {
    let _guard = config::logging::configure_tracing()?;

    HttpServer::new(|| App::new().service(root))
        .bind((config::server::get_host(), config::server::get_port()))?
        .run()
        .await?;

    Ok(())
}
