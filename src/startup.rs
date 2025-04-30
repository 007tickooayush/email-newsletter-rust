use std::net::TcpListener;
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use actix_web::middleware::Logger;
use sqlx::{PgPool};
use crate::routes::{health_check, subscribe};

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {

    // using web::Data to wrap the connection in smart pointer(Arc)
    // as App required the app_data to implement Clone trait for "T"
    // and in Arc<T> T is clonable, no matter what T is
    let connection = web::Data::new(db_pool);

    let server = HttpServer::new(move || {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .wrap(Logger::default())
            .app_data(connection.clone())
    })
        .listen(listener)?
        .run();
    // No .await here
    Ok(server)
}