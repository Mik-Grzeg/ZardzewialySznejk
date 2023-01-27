use actix_web::{get, post, error, web, App, Result};
use crate::game::Direction;

#[get("/snake")]
async fn get_game_state() -> Result<String> {
    todo!()
}

#[post("/snake/:{direction}")]
async fn post_direction_command(path: web::Path<String>) -> Result<String> {
    let direction = Direction::try_from(path.into_inner())
        .map_err(|e| error::ErrorBadRequest(e))?;



    todo!()
}
