mod config;
use actix_web::{get, App, HttpServer, Responder};
use anyhow::Result as AnyhowResult;
use tracing_actix_web::TracingLogger;

#[get("/")]
async fn root() -> impl Responder {
    tracing::info!("Root");
    "I am root"
}

#[tokio::main]
async fn main() -> AnyhowResult<()> {
    let _guard = config::logging::configure_tracing()?;

    HttpServer::new(|| App::new().wrap(TracingLogger::default()).service(root))
        .bind((
            config::server::get_host().as_ref(),
            config::server::get_port(),
        ))?
        .run()
        .await?;

    Ok(())
}
