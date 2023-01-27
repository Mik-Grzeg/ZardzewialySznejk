use snake::game::start_game;

use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

const FPS: f32 = 0.5;

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
    // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
    // will be written to stdout.
    .with_max_level(Level::TRACE)
    // completes the builder.
    .finish();

    tracing::subscriber::set_global_default(subscriber)
    .expect("setting default subscriber failed");

    start_game(FPS).await;
}
