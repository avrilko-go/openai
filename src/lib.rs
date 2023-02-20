use std::convert::Infallible;
use axum::response::sse::Event;
use reqwest::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, CACHE_CONTROL, CONNECTION, CONTENT_TYPE, HeaderMap};
use serde::{Serialize, Deserialize};
use tokio_stream::StreamExt;

#[derive(Serialize, Deserialize)]
struct PostData {
    model: String,
    max_tokens: usize,
    stream: bool,
    prompt: String,
}


impl Default for PostData {
    fn default() -> Self {
        Self {
            model: "text-davinci-003".to_string(),
            max_tokens: 2048,
            stream: true,
            prompt: "hello".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct RecvData {
    id: String,
    object: String,
    created: usize,
    choices: Vec<TrueData>,
    model: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct TrueData {
    text: String,
    index: usize,
    finish_reason: Option<String>,
}


pub async fn send_request(prompt: String, sender:tokio::sync::mpsc::Sender<Result<Event,Infallible>>) {
    let client = Client::new();
    let mut post_data = PostData::default();
    post_data.prompt = prompt;
    let mut headers = HeaderMap::new();
    headers.insert(CACHE_CONTROL, "no-cache".parse().unwrap());
    headers.insert(ACCEPT, "text/event-stream".parse().unwrap());
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    headers.insert(CONNECTION, "keep-alive".parse().unwrap());
    headers.insert(
        AUTHORIZATION,
        "Bearer "
            .parse()
            .unwrap(),
    );
    let res = client
        .post("https://api.openai.com/v1/completions")
        .json(&post_data)
        .headers(headers)
        .send()
        .await
        .unwrap();
    let mut a = res.bytes_stream();

    while let Some(Ok(v)) = a.next().await {
        let s = String::from_utf8(v.to_vec()).unwrap();
        let c = s.trim_end_matches("\n\n").trim_start_matches("data: ");
        if c == "[DONE]".to_string() {
            sender.send(Ok(Event::default().data(c))).await.unwrap();
            return;
        }
        if let Ok(value) = serde_json::from_str::<RecvData>(c) {
            sender.send(Ok(Event::default().data(value.choices[0].text.clone()))).await.unwrap();
        } else {
            println!("{:?}",s);
            sender.send(Ok(Event::default().data("[ERROR]"))).await.unwrap();
            return;
        }
    }
}