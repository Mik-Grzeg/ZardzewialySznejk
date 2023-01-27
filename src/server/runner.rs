use actix_web::{web, App, HttpResponse, HttpServer};

struct ServerSettings {
    port: u16,
}

async fn serve(_settings: ServerSettings) {
    HttpServer::new(|| App::new().route("/snake", web::get().to(HttpResponse::Ok))).workers(4);
}
