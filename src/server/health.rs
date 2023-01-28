use actix_web::{get};

#[get("/healthz")]
async fn healthy() -> String {
    "Healthy".to_owned()
}
