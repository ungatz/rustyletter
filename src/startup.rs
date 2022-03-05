use actix_web::{web, App, HttpResponse, HttpServer};
use actix_web::dev::Server;
use std::net::TcpListener;
use sqlx::PgPool;
use crate::routes::{
     health_check, subscribe
};

pub fn run(listner: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    // Wrap connection with smart pointer
    let db_pool = web::Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(db_pool.clone())
    })
    .listen(listner)?
    .run();
    Ok(server)
}
