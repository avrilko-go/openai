use std::collections::HashMap;
use std::convert::Infallible;
use std::io;
use std::net::SocketAddr;
use std::time::Duration;
use axum::{Router};
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Sse};
use axum::response::sse::{Event, KeepAlive};
use axum::routing::{get, get_service};
use futures::stream::{Stream, self};
use tokio_stream::StreamExt;
use tower_http::services::ServeDir;
use chat::send_request;

#[tokio::main]
async fn main() {
    let serve_dir_from_dist = get_service(ServeDir::new("dist")).handle_error(handle_error);

    let app = Router::new().nest_service("/",serve_dir_from_dist).route("/sse", get(handle_sse));
    let addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();
    axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
}

async fn handle_sse(Query(params):Query<HashMap<String,String>>) -> Sse<impl Stream<Item=Result<Event, Infallible>>> {
    let content = params.get("content").unwrap().clone();

    let (tx, mut rx) = tokio::sync::mpsc::channel::<Result<Event, Infallible>>(1024);


    let task1 = async move {
        send_request(content,tx).await;
    };

    tokio::task::spawn(task1);


    let stream = async_stream::stream! {
        while let Some(item) = rx.recv().await {
            yield item;
        }
    };
    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn handle_error(_err: io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}