use actix_web::{web, App, HttpResponse, HttpServer};

struct ServerSettings {
    port: u16
}

async fn serve(settings: ServerSettings) {
    HttpServer::new(|| App::new().route("/snake", web::get().to(HttpResponse::Ok))).workers(4);
}

