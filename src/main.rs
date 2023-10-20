#[macro_use]
extern crate diesel;
use actix::*;
use actix_cors::Cors;
use actix_files::Files;
use actix_web::{web, http, App, HttpServer};
use diesel::{
    prelude::*,
    r2d2::{self, ConnectionManager}
}
mod db;
mod models;
mod routes;
mod schema;
mod server;
mod session;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //start the server
    let server = server::ChatServer::new().start();

    //specifying path to SQLite DB file path for connectionManager
    let conn_spec = "chat.db";

    //instantiating the manager, using conn_spec to determine where to connect to
    let manager = ConnectionManager::<SqliteConnection>::new(conn_spec);

    //instantiating the r2d2 connection pool, throws error on failure
    let pool = r2d2::Pool::builder().build(manager).expect("Failed to create pool.");

    //server info
    let server_addr: &str = "127.0.0.1";
    let server_port: i32 = 8080;

    let app = HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("https://localhost:3000")
            .allowed_origin("https://localhost:8000")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http:header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);
        App::new()
            .app_data(web::Data::new(server.clone()))
            .app_data(web::Data::new(pool.clone()))
            .wrap(cors)
            .service(web::resource("/").to(routes::index))
            .route("/ws", web::get().to(routes::chat_server))
            .service(routes::create_user)
            .service(routes::get_user_by_id)
            .service(routes::get_user_by_phone)
            .service(routes::get_conversation_by_id)
            .service(routes::get_rooms)
            .service(Files::new("/", "./static"))
    })


}
