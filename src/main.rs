use std::{net::{IpAddr, Ipv4Addr, SocketAddr}, error::Error};

use axum::{
    response::IntoResponse,
    routing::get,
    Router, Server,
};
use tower_http::services::ServeDir;

async fn hello_world() -> impl IntoResponse {
    String::from("Hello, world!")
}

fn create_router() -> Router {
    Router::new()
        .route("/hello", get(hello_world))
        .nest_service("/serve_dir", ServeDir::new("static"))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app = create_router();

    const PORT: u16 = 8080;
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), PORT);
    Server::bind(&addr).serve(app.into_make_service()).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use axum_test::TestServer;

    #[tokio::test]
    async fn hello() {
        let server = TestServer::new(create_router().into_make_service()).unwrap();

        let response = server.get("/hello").await;

        assert_eq!(response.text(), "Hello, world!");
    }

    #[tokio::test]
    async fn serve_dir_basic_html() -> Result<(), Box<dyn Error>> {
        let server = TestServer::new(create_router().into_make_service()).unwrap();

        let response = server.get("/serve_dir/basic.html").await;

        let expected = tokio::fs::read("static/basic.html").await?;
        assert_eq!(response.bytes(), expected);
        Ok(())
    }

    #[tokio::test]
    async fn serve_dir_scout_webp() -> Result<(), Box<dyn Error>> {
        let server = TestServer::new(create_router().into_make_service()).unwrap();

        let response = server.get("/serve_dir/scout.webp").await;

        let expected = tokio::fs::read("static/scout.webp").await?;
        assert_eq!(response.bytes(), expected);
        Ok(())
    }
}