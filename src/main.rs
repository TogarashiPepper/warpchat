#![feature(never_type)]
#![feature(type_alias_impl_trait)]

use futures_util::StreamExt;
use rocket::Store;
use std::{
    convert::Infallible,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;
use warp::{sse::Event, Filter};
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    use diesel::prelude::*;
    use rocket::schema::posts::dsl::*;

    dotenv().ok();

    let password = std::env::var("PASSWORD").unwrap();

    let conn = rocket::establish_connection();
    let cors = warp::cors()
        .allow_any_origin();

    let r = posts
        .limit(120)
        .order(id.desc())
        .load::<rocket::models::Post>(&conn)
        .expect("Error loading posts");
    let r = r.iter();
    let r = r.map(|p| p.msg.clone()).collect::<Vec<String>>();

    let store = rocket::Store {
        state: Arc::new(Mutex::new(conn)),
        vec_state: Arc::new(Mutex::new(r)),
    };
    let store_filter = warp::any().map(move || store.clone());

    let files = warp::fs::dir("public");

    let send = warp::path!("sendfoo" / String / String)
        .and(warp::get())
        .and(store_filter.clone())
        .and_then(rocket::handle_send)
        .with(&cors);

    let ticks = warp::path(format!("ticks_{}", password))
        .and(warp::get())
        .and(store_filter.clone())
        .map(|s: Store| {
            // create server event source
            let interval = interval(Duration::from_secs(3));
            let stream = IntervalStream::new(interval);
            let event_stream = stream.map(move |_| -> Result<Event, Infallible> {
                let c = s.vec_state.lock().unwrap();
                let results = c.join("\n");

                Ok(warp::sse::Event::default().data(results))
            });
            // reply using server-sent events
            warp::sse::reply(event_stream)
        })
        .with(&cors);
    warp::serve(send.or(ticks).or(files))
        .run(([127, 0, 0, 1], 3030))
        .await;
}

// pub fn handle_ticks(store: Store) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
//     use diesel::prelude::*;
//     use rocket::schema::posts::dsl::*;

//     warp::path("ticks").and(warp::get()).map(|| {
//         let mut counter: u64 = 0;
//         let interval = interval(Duration::from_secs(1));
//         let stream = IntervalStream::new(interval);
//         let event_stream = stream.map(move |_| {
//             let results = posts
//                 .limit(120)
//                 .order(id.desc())
//                 .load::<models::Post>(&*conn)
//                 .expect("Error loading posts");
//             let results = results.iter();
//             let results = results.map(|p| p.msg.clone()).collect::<Vec<String>>();
//             let results = results.join("\n");

//             Ok(warp::sse::Event::default().data(results))
//         });
//         // reply using server-sent events
//         warp::sse::reply(event_stream)
//     })
// }
