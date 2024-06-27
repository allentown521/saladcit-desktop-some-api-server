use actix_web::{App, HttpServer};
use handler::*;
mod handler;
mod utils;

pub static AFDIAN_USER_ID: &str = "AFDIAN_USER_ID";
pub static AFDIAN_TOKEN: &str = "AFDIAN_TOKEN";
pub static ALIPAN_CLIENTID: &str = "ALIPAN_CLIENTID";
pub static ALIPAN_SECRET: &str = "ALIPAN_SECRET";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let port = match args.first() {
        Some(port) => port,
        None => "8001",
    };
    let port = port.parse::<u16>().unwrap_or(8001);
    HttpServer::new(|| {
        App::new()
            .service(afdian)
            .service(dict)
            .service(ali_qrcode)
            .service(ali_access_token)
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
