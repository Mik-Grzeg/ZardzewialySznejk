use crate::game::Direction;
use actix_web::{error, get, post, web, Result};

#[get("/snake")]
async fn get_game_state() -> Result<String> {
    todo!()
}

#[post("/snake/:{direction}")]
async fn post_direction_command(path: web::Path<String>) -> Result<String> {
    let _direction =
        Direction::try_from(path.into_inner()).map_err(error::ErrorBadRequest)?;

    todo!()
}
