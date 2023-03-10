use std::sync::RwLock;

use crate::game::{movement::OrderMove, Board, Direction};
use actix_web::{dev::HttpServiceFactory, error, services, web, Result};
use std::sync::Arc;
use tracing::error;

pub fn snake_service<T>(
    board: Arc<RwLock<Board>>,
    move_manager: Arc<RwLock<T>>,
) -> impl HttpServiceFactory
where
    T: OrderMove + 'static,
{
    services![web::scope("/snake")
        .service(
            web::resource("")
                .app_data(web::Data::new(board))
                .route(web::get().to(get_game_state))
        )
        .service(
            web::resource("/{direction}")
                .app_data(web::Data::new(move_manager))
                .route(
                    web::post().to::<_, (actix_web::web::Path<String>, web::Data<Arc<RwLock<T>>>)>(
                        post_direction_command
                    )
                )
        )]
}

async fn get_game_state(board: web::Data<Arc<RwLock<Board>>>) -> Result<String> {
    let board = board.read().unwrap();
    let mut out = String::new();

    board.get_board(&mut out).map_err(|e| {
        error!("Writing board to str failed: {}", e);
        error::ErrorInternalServerError(e)
    })?;

    Ok(out)
}

async fn post_direction_command(
    path: web::Path<String>,
    move_manager: web::Data<Arc<RwLock<impl OrderMove>>>,
) -> Result<&'static str> {
    let direction = Direction::try_from(path.into_inner()).map_err(error::ErrorBadRequest)?;

    let move_manager = move_manager.read().unwrap();
    move_manager
        .issue_move(direction)
        .map_err(error::ErrorInternalServerError)?;

    Ok("")
}
