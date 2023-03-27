use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::{response::IntoResponse, routing::get, Router};

async fn hello_world() -> impl IntoResponse {
    String::from("Hello, world!")
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/hello", get(hello_world));

    const PORT: u16 = 8080;
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), PORT);
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}