use snake::game::new_game;
use snake::game::movement::OrderMove;
use snake::server;

use tracing::{Level, info};
use tracing_subscriber::{FmtSubscriber, EnvFilter, Registry, layer::Layered, filter::LevelFilter, fmt::format::{DefaultFields, Format}};

const FPS: f32 = 0.5;

type TracingSub = FmtSubscriber<DefaultFields, Format, tracing_subscriber::reload::Layer<EnvFilter, Layered<tracing_subscriber::fmt::Layer<Registry>, Registry>>>;

fn init_tracing() -> TracingSub {
    FmtSubscriber::builder()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy()
        )
        .with_filter_reloading()
        .finish()
}

#[tokio::main]
async fn main() {
    let subscriber = init_tracing();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    loop {
        let (mut game, move_orderer, board) = new_game(FPS);
        let board = game.board.clone();

        tokio::select! {
            _ = game.start() => { }
            _ = server::run(move_orderer, Arc::clone(&board)) => {
                info!("HTTP server shutdown");
                break;
            }
        }
    }
}
