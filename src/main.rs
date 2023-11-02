use actix_web::{App, HttpServer};
use handler::*;
mod handler;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(afdian).service(dict))
        .bind(("127.0.0.1", 8001))?
        .run()
        .await
}
