use actix_web::{App, HttpServer};
use handler::*;
mod handler;
mod utils;

pub static AFDIAN_USER_ID: &str = "AFDIAN_USER_ID";
pub static AFDIAN_TOKEN: &str = "AFDIAN_TOKEN";
pub static ALIPAN_CLIENTID: &str = "ALIPAN_CLIENTID";
pub static ALIPAN_SECRET: &str = "ALIPAN_SECRET";
pub static PORT: u32 = 8001;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(afdian)
            .service(dict)
            .service(ali_qrcode)
            .service(ali_access_token)
    })
    .bind(("127.0.0.1", PORT))?
    .run()
    .await
}
