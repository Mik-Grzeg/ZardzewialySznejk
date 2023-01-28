use actix_web::{web, App, HttpResponse, HttpServer};
use tracing_actix_web::TracingLogger;
use super::handlers::snake_service;
use super::health::healthy;
use crate::game::movement::OrderMove;
use crate::game::Board;
use super::state::AppState;
use std::sync::{Arc, RwLock};
use tracing::info;

const host: &str = "0.0.0.0";
const port: u16 = 8080;

pub async fn run<T>(move_manager: Arc<RwLock<T>>, board: Arc<RwLock<Board>>) -> std::io::Result<()>
where
    T: OrderMove + 'static
{
    info!("Starting web server on {}:{}", host, port);

    HttpServer::new(
        move || {
            App::new()
                .wrap(TracingLogger::default())
                .service(snake_service(Arc::clone(&board), Arc::clone(&move_manager)))
                .service(healthy)
        })
        .bind((host, port))?
        .run()
        .await
}
