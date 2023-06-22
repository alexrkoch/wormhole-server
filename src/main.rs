mod config;
mod game;

use std::sync::Mutex;

use actix_web::{body::BoxBody, web, App, HttpResponse, HttpServer};
use anyhow::Result as AnyhowResult;
use tokio::sync::mpsc::Sender;
use tracing_actix_web::TracingLogger;

use crate::game::{RoomDeletionHandler, RoomId, RoomRegistry};

async fn get_rooms(registry: web::Data<Mutex<RoomRegistry>>) -> web::Json<Vec<String>> {
    let mut room_registry = registry.lock().unwrap();
    let room_ids = room_registry.list_active_rooms();
    web::Json(room_ids)
}

async fn create_room(
    registry: web::Data<Mutex<RoomRegistry>>,
    sender: web::Data<Sender<RoomId>>,
) -> HttpResponse {
    // TODO (mitch): Graceful handling of lock acquisition
    // https://github.com/alexrkoch/wormhole-server/issues/10
    let mut room_registry = registry.lock().unwrap();
    let create_room_result = room_registry.create_room(sender.get_ref().clone());

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
    cfg.service(
        web::resource("/rooms/")
            .route(web::post().to(create_room))
            .route(web::get().to(get_rooms)),
    );
}

#[tokio::main]
async fn main() -> AnyhowResult<()> {
    let _guard = config::logging::configure_tracing()?;
    let room_registry = web::Data::new(Mutex::new(RoomRegistry::new()));

    let (mut handler, sender) =
        RoomDeletionHandler::new_with_registry(room_registry.clone().into_inner());
    let sender = web::Data::new(sender);

    let room_deletion_handle = handler.watch();

    let server_handle = HttpServer::new(move || {
        App::new()
            .app_data(room_registry.clone())
            .app_data(sender.clone())
            .service(
                web::scope("api/v1")
                    .wrap(TracingLogger::default())
                    .configure(configure_api_scope),
            )
    })
    .bind((
        config::server::get_host().as_ref(),
        config::server::get_port(),
    ))?
    .run();

    futures::join!(room_deletion_handle, server_handle);

    Ok(())
}
