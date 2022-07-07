#![feature(never_type)]
#[macro_use]
extern crate diesel;

pub mod models;
pub mod routes;
pub mod schema;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use models::{NewPost, Post};
use urlencoding::decode;
use std::{
    env,
    sync::{Arc, Mutex},
};
use warp::http;

macro_rules! warp_reply {
    ($x:tt, $y:ident) => {
        warp::reply::with_status(String::from($x), http::StatusCode::$y)
    };
}

pub fn establish_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

pub fn create_post<'a>(conn: &PgConnection, msg: &'a str) -> Post {
    use schema::posts;

    let new_post = NewPost { msg };

    diesel::insert_into(posts::table)
        .values(&new_post)
        .get_result(conn)
        .expect("Error saving new post")
}

pub fn clear_table(conn: &PgConnection) {
    use self::schema::posts::dsl::*;

    diesel::delete(posts)
        .execute(conn)
        .expect("Error deleting posts");
    let p = NewPost {
        msg: "[Test]: test",
    };
    diesel::insert_into(posts)
        .values(&p)
        .execute(conn)
        .expect("Error inserting first post");
}

pub fn self_clear_table(conn: &PgConnection, name: String) {
    use self::schema::posts::dsl::*;

    diesel::delete(posts.filter(msg.like(format!("[{}]: %", &name))))
        .execute(conn)
        .expect("Error deleting posts");
}

pub fn delete_post(conn: &PgConnection, pat: String) {
    use self::schema::posts::dsl::*;

    diesel::delete(posts.filter(msg.like(pat)))
        .execute(conn)
        .expect("Error deleting posts");
}

// pub async fn giphy_search(query: &str) -> Result<String, String> {
//     use giphy::v1::gifs::SearchRequest;
//     use giphy::v1::r#async::*;

//     let api_key = std::env::var("GIPHY_API_KEY")
//         .unwrap_or_else(|e| panic!("Error retrieving env variable: {:?}", e));
//     let client = reqwest::Client::new();
//     let api = AsyncApi::new(api_key, client);

//     let response = SearchRequest::new(query)
//         .with_limit(1)
//         .send_to(&api)
//         .await
//         .unwrap();

//     match response.data.get(0) {
//         Some(v) => Ok(v.url.clone()),
//         None => Err(format!("No GIFs Found with search query: '{}'", query)),
//     }
// }

#[derive(Clone)]
pub struct Store {
    pub state: Arc<Mutex<PgConnection>>,
    pub vec_state: Arc<Mutex<Vec<String>>>
}

pub async fn handle_send(
    message: String,
    authtoken: String,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    use diesel::prelude::*;
    use schema::posts::dsl::*;

    if !authtoken.starts_with(&std::env::var("PASSWORD").unwrap()) {
        return Ok(warp_reply!("no", NOT_FOUND));
    }

    let message = decode(message.as_str()).unwrap();
    let authtoken = decode(authtoken.as_str()).unwrap();

    let conn = store.state.lock().unwrap();
    let mut msg_vec = store.vec_state.lock().unwrap();

    if message == "null" {
    } else if message.starts_with("!delete") {
        let payload = message.to_owned()[8..].to_owned();
        delete_post(&conn, format!("[{}]: {}", &authtoken[21..], payload));
    } else if message.starts_with("!adminclear") {
        clear_table(&conn);
    } else if message.starts_with("!clear") {
        self_clear_table(&conn, (authtoken[21..]).to_string());
    } else if message.starts_with("!help") {
        let help_text = include_str!("help.txt");
        return Ok(warp_reply!(help_text, OK));
    } else {
        create_post(&conn, &format!("[{}]: {}", &authtoken[21..], message));
    }

    let results = posts
        .limit(120)
        .order(id.desc())
        .load::<models::Post>(&*conn)
        .expect("Error loading posts");
    let results = results.iter();
    let results = results.map(|p| p.msg.clone()).collect::<Vec<String>>();

    *msg_vec = results.clone();

    let results = results.join("\n");

    Ok(warp_reply!(results, OK))
}
