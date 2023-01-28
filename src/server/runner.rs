use super::handlers::snake_service;
use super::health::healthy;

use crate::game::movement::OrderMove;
use crate::game::Board;
use actix_web::{App, HttpServer};
use std::sync::{Arc, RwLock};
use tracing::info;
use tracing_actix_web::TracingLogger;

const HOST: &str = "0.0.0.0";
const PORT: u16 = 8080;

pub async fn run<T>(move_manager: Arc<RwLock<T>>, board: Arc<RwLock<Board>>) -> std::io::Result<()>
where
    T: OrderMove + 'static,
{
    info!("Starting web server on {}:{}", HOST, PORT);

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .service(snake_service(Arc::clone(&board), Arc::clone(&move_manager)))
            .service(healthy)
    })
    .bind((HOST, PORT))?
    .run()
    .await
}
