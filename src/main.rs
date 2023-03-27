use std::{
    error::Error,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use axum::{response::IntoResponse, routing::get, Router, Server};
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
    use paste::paste;

    #[tokio::test]
    async fn hello() {
        let server = TestServer::new(create_router().into_make_service()).unwrap();

        let response = server.get("/hello").await;

        assert_eq!(response.text(), "Hello, world!");
    }

    macro_rules! test_static_files {
        ($prefix:ident, $base_path:expr) => {
            paste! {
            #[tokio::test]
                async fn [<$prefix _basic_html>]() -> Result<(), Box<dyn Error>> {
                    let server = TestServer::new(create_router().into_make_service()).unwrap();

                    let path = format!("{}/basic.html", $base_path);
                    let response = server.get(&path).await;

                    let expected = tokio::fs::read("static/basic.html").await?;
                    assert_eq!(response.bytes(), expected);
                    Ok(())
                }
            }

            paste! {
                #[tokio::test]
                async fn [<$prefix _scout_webp>]() -> Result<(), Box<dyn Error>> {
                    let server = TestServer::new(create_router().into_make_service()).unwrap();

                        let path = format!("{}/scout.webp", $base_path);
                    let response = server.get(&path).await;

                    let expected = tokio::fs::read("static/scout.webp").await?;
                    assert_eq!(response.bytes(), expected);
                    Ok(())
                }
            }
        };
    }

    test_static_files!(serve_dir, "/serve_dir");
}
