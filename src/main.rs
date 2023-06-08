mod config;
use actix_web::{web, App, HttpServer, Responder};
use anyhow::Result as AnyhowResult;
use tracing_actix_web::TracingLogger;

async fn api_root() -> impl Responder {
    "I am root"
}

fn configure_api_scope(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/").route(web::get().to(api_root)));
}

#[tokio::main]
async fn main() -> AnyhowResult<()> {
    let _guard = config::logging::configure_tracing()?;

    HttpServer::new(|| {
        App::new().service(
            web::scope("api/v1")
                .wrap(TracingLogger::default())
                .configure(configure_api_scope),
        )
    })
    .bind((
        config::server::get_host().as_ref(),
        config::server::get_port(),
    ))?
    .run()
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http, test};

    #[actix_web::test]
    async fn test_get_api_v1_root_is_ok() {
        let req = test::TestRequest::default().to_http_request();
        let responder = api_root().await;
        let response = responder.respond_to(&req);
        assert_eq!(response.status(), http::StatusCode::OK);
    }
}
