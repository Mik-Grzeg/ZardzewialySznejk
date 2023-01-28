use std::sync::RwLock;

use crate::game::{Direction, Board, movement::OrderMove};
use actix_web::{error, get, post, web, Result, services, dev::{HttpServiceFactory}};
use std::sync::Arc;
use tracing::{error, info};


pub fn snake_service<T>(board: Arc<RwLock<Board>>, move_manager: T) -> impl HttpServiceFactory
where
    T: OrderMove + 'static
{
    services![
        web::scope("/snake")
            .service(
                web::resource("")
                    .app_data(web::Data::new(board))
                    .route(web::get().to(get_game_state))
            )
            .service(
                web::resource("/{direction}")
                    .app_data(web::Data::new(move_manager))
                    .route(web::post().to::<_, (actix_web::web::Path<String>, web::Data<T>)>(post_direction_command))
            )
    ]
}

async fn get_game_state(board: web::Data<Arc<RwLock<Board>>>) -> Result<String> {
    let board = board.read().unwrap();
    let mut out = String::new();

    board.get_board(&mut out)
        .map_err(|e| {
            error!("Writing board to str failed: {}", e);
            error::ErrorInternalServerError(e)
        })?;

    Ok(out)
}

async fn post_direction_command(path: web::Path<String>, move_manager: web::Data<impl OrderMove>) -> Result<&'static str> {
    let direction =
        Direction::try_from(path.into_inner()).map_err(error::ErrorBadRequest)?;

    move_manager.issue_move(direction).await
        .map_err(|e| error::ErrorInternalServerError(e))?;

    Ok("")
}
