use std::convert::Infallible;
use std::net::SocketAddr;
use std::time::Duration;
use axum::{Router};
use axum::response::Sse;
use axum::response::sse::{Event, KeepAlive};
use axum::routing::get;
use futures::stream::{Stream, self};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/sse", get(handle_sse));
    let addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();
    axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
}

async fn handle_sse() -> Sse<impl Stream<Item=Result<Event, Infallible>>> {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Result<Event, Infallible>>(1024);

    let stream = async_stream::stream! {
        while let Some(item) = rx.recv().await {
            yield item;
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}