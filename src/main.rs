use axum::{Router, AddExtensionLayer};
use axum::extract::{Extension};
use axum::http::StatusCode;
use axum::routing::get;
use std::thread;
use crossbeam_channel::bounded;
use log::error;
use metrics::{describe_counter, increment_counter};
use metrics_exporter_prometheus::PrometheusBuilder;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let http_port = std::env::var("HTTP_PORT").expect("HTTP_PORT environment variable");
    let addr = format!("0.0.0.0:{}", http_port);

    let builder = PrometheusBuilder::new();
    builder
        .install()
        .expect("failed to install Prometheus recorder");
    describe_counter!("count", "number of access");

    let (sender, receiver) = bounded(0);

    thread::spawn(move || count_up(sender));

    let app = Router::new()
        .route("/", get(index))
        .layer(AddExtensionLayer::new(receiver));

    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn count_up(sender:crossbeam_channel::Sender<u64>){
    let mut count = 0;
    loop {
        match sender.send(count){
            Ok(_) => {
                count += 1;
                increment_counter!("count");
            },
            Err(e) => {
                error!("failed to send message {}", e);
            },
        }
    }
}

async fn index(Extension(receiver): Extension<crossbeam_channel::Receiver<u64>>) -> Result<String, StatusCode> {
    match receiver.recv(){
        Ok(count) => {
            Ok(count.to_string())
        },
        Err(e ) => {
            error!("failed to receive message {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        },
    }
}