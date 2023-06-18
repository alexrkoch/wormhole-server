mod config;
mod game;

use std::sync::Mutex;

use crate::game::RoomRegistry;

use actix_web::{body::BoxBody, web, App, HttpResponse, HttpServer};
use anyhow::Result as AnyhowResult;
use tracing_actix_web::TracingLogger;

async fn create_room(state: web::Data<SharedAppState>) -> HttpResponse {
    // TODO (mitch): Graceful handling of lock acquisition
    // https://github.com/alexrkoch/wormhole-server/issues/10
    let mut room_registry = state.room_registry.lock().unwrap();
    let create_room_result = room_registry.create_room();

    match create_room_result {
        Err(e) => HttpResponse::InternalServerError()
            .message_body(BoxBody::new(format!("{e:?}")))
            .unwrap(),
        Ok(room_id) => HttpResponse::Created()
            .insert_header(("LOCATION", format!("/ws/{room_id}")))
            .finish(),
    }
}

fn configure_api_scope(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/rooms/").route(web::post().to(create_room)));
}

struct SharedAppState {
    room_registry: Mutex<RoomRegistry>,
}

#[tokio::main]
async fn main() -> AnyhowResult<()> {
    let _guard = config::logging::configure_tracing()?;
    let state = web::Data::new(SharedAppState {
        room_registry: Mutex::new(RoomRegistry::new()),
    });

    HttpServer::new(move || {
        App::new().app_data(state.clone()).service(
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
