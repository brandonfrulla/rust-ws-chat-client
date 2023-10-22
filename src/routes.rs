use std::time::Instant;
use actix::*;
use actix_files::NamedFile;
use actix_web::{get, post, web, Error, HttpRequest, HttpResponse, Responder};
use actix_web_actors::ws;
use diesel::{
    prelude::*,
    r2d2::{self, ConnectionManager},
};
use serde_json::json;
use uuid::Uuid;
use crate::db;
use crate::models;
use crate::server;
use crate::session;
type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

pub async fn index() -> impl Responder {
    NamedFile::open_async("./static/index.html").await.unwrap()
}

//defining our server itself
pub async fn chat_server(
    req: HttpRequest,                           // HTTP req obj
    stream: web::Payload,                       // The payload stream for WS comms
    pool: web::Data<DbPool>,                    // async db connection pool
    srv: web::Data<Addr<server::ChatServer>>,   // host addr
) -> Result<HttpResponse, Error> {              // we'll either respond with Http or error
    ws::start(
        session::WsChatSession {
            id: 0,
            hb: Instant::now(),                 // heartbeat timestamp
            room: "main".to_string(),
            name: None,
            addr: srv.get_ref().clone(),
            db_pool: pool,
        },
        &req,
        stream
    ).expect("Couldn't start chat server.") // if ws::start fails
}

#[post("/users/create")]                        // Macro indicates that this will be reiggered via POST req to /users/create
pub async fn create_user(                        
    pool: web::Data<DbPool>,
    form: web::Json<models::NewUser>,
) -> Result<HttpResponse, Error> {
    let user = web::block(move || {
        let mut conn = pool.get()?;
        db::insert_new_user(&mut conn, &form.username, &form.phone)
    })
    .await?
    .map_err(actix_web::error::ErrorUnprocessableEntity)?;
    Ok(HttpResponse::Ok().join(user))
}

#[get("/users/{user_id}")]                       // Macro indicates that this will be reiggered via GET req to /users/{Uuid}
pub async fn get_user_by_id(
    pool: web::Data<DbPool>,
    id: web::Path{Uuid},
) -> Result<HttpResponse, Error> {
    let user_id = id.to_owned();                // This line takes ownership of the id by cloning it
    let user = web::block(move || {
        let mut conn = pool.get()?;
        db::find_user_by_id(&mut conn, user_id)
    })
    .await?
    .map_err(actix::error::ErrorInternalServerError)?;

    if let Some(user) = user {
        Ok(HttpResponse::Ok().json(user))
    } else {
        let res = HttpResponse::NotFound().body(
            json!({
                "error": 404,
                "message": format!("No user found with phone: {id}")
            })
            .to_string(),
        );
        Ok(res)
    }
}

#[get("/conversations/{uid}")]
pub async fn get_conversation_by_id(
    pool: web::Data<DbPool>,
    uid: web::Path<Uuid>,
) -> Result <HttpResponse, Error> {
    let room_id = uid.to_owned();
    let conversations = web::block(move ||{
        let mut conn = pool.get()?;
        db::get_conversation_by_room_uid(&mut conn, room_id)
    })
    .await?
    map_err(actix::error::ErrorInternalServerError)?;
    if let Some(data) = conversations {
        Ok(HttpResponse::Ok().json(data))
    } else {
        let res = HttpResponse::NotFound().body({
            json!({
                "error": 404,
                "message": format!("No conversation with room_id: {room_id}")
            })
            .to_string(),
        });
        Ok(res)
    }
}
