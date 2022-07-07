use std::time::Duration;

use futures_util::StreamExt;
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;
use warp::{Filter, sse::Event};

type Never = std::convert::Infallible;

pub fn sum(a: u32, b: u32) -> String {
    format!("{} + {} = {}", a, b, a + b)
}

fn sse_counter(counter: u64) -> Result<Event, Never> {
    Ok(warp::sse::Event::default().data(counter.to_string()))
}

pub fn ticks() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("ticks").and(warp::get()).map(|| {
        let mut counter: u64 = 0;
        let interval = interval(Duration::from_secs(1));
        let stream = IntervalStream::new(interval);
        let event_stream = stream.map(move |_| {
            counter += 1;
            sse_counter(counter)
        });
        // reply using server-sent events
        warp::sse::reply(event_stream)
    })
}

