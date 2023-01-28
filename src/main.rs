use snake::game::new_game;

use tracing_subscriber::{
    filter::LevelFilter,
    fmt::format::{DefaultFields, Format},
    layer::Layered,
    EnvFilter, FmtSubscriber, Registry,
};

const FPS: f32 = 10.0;

type TracingSub = FmtSubscriber<
    DefaultFields,
    Format,
    tracing_subscriber::reload::Layer<
        EnvFilter,
        Layered<tracing_subscriber::fmt::Layer<Registry>, Registry>,
    >,
>;

fn init_tracing() -> TracingSub {
    FmtSubscriber::builder()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with_filter_reloading()
        .finish()
}

#[tokio::main]
async fn main() {
    let subscriber = init_tracing();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    new_game(FPS).await;
}
