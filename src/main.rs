use snake::game::new_game;
use snake::game::movement::OrderMove;
use snake::server;
use std::sync::Arc;

use tracing::{Level, info};
use tracing_subscriber::{FmtSubscriber, EnvFilter, Registry, layer::Layered, filter::LevelFilter, fmt::format::{DefaultFields, Format}};

const FPS: f32 = 10.0;

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

// async fn start_components() {
//     let (mut game, move_orderer) = new_game(FPS);
//     let board = Arc::clone(&game.board);
// }

#[tokio::main]
async fn main() {
    let subscriber = init_tracing();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    new_game(FPS).await;
    // loop {
    //     let (mut game, move_orderer) = new_game(FPS);

    //     let (command_sender, command_recv) = mpsc::channel(MOVE_COMMAND_CHANNEL_SIZE);
    //     let game = Game::new(command_recv.into(), fps);
    //     let order_move = MoveCommandIssuer::from(command_sender);

    //     let board = Arc::clone(&game.board);

    //     tokio::select! {
    //         _ = game.start() => {
    //             info!("Game finished");
    //             continue;
    //         }
    //         _ = server::run(&move_orderer, board) => {
    //             info!("HTTP server shutdown");
    //             break;
    //         }
    //     }
    // }
}
