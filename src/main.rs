use std::{
    error::Error,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
    pin::{pin, Pin},
    task::{Context, Poll},
};

use axum::{
    body::Bytes,
    extract::Path,
    response::{IntoResponse, Response},
    routing::get,
    Router, Server,
};
use bytes::BytesMut;
use futures_core::stream::Stream;
use http_body::Body;
use hyper::{HeaderMap, StatusCode};
use pin_project::pin_project;
use tokio::{
    fs::File,
    io,
};
use tokio_stream::StreamExt;
use tokio_util::io::ReaderStream;
use tower_http::services::ServeDir;

async fn hello_world() -> impl IntoResponse {
    String::from("Hello, world!")
}

/// Performs a naive read of the file at the given path into memory, then returns that data.
///
/// Returns NOT_FOUND on any error.
async fn read_file(Path(relative_path): Path<String>) -> Result<impl IntoResponse, StatusCode> {
    let mut path = PathBuf::from("static");
    path.push(&relative_path);
    let result = tokio::fs::read(&path).await;
    match result {
        Ok(bytes) => Ok(bytes),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

/// A streaming body response, based on AsyncReadBody from tower.
#[pin_project]
struct BodyStream(#[pin] pub ReaderStream<File>);

impl Body for BodyStream {
    type Data = Bytes;
    type Error = io::Error;

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        self.project().0.poll_next(cx)
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(None))
    }
}

/// Opens the file with tokio, then uses ReaderStream from tokio-util to build the response.
///
/// Returns NOT_FOUND on any error.
async fn stream_to_body(
    Path(relative_path): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut path = PathBuf::from("static");
    path.push(&relative_path);
    let file = tokio::fs::File::open(path)
        .await
        .map_err(|_err| StatusCode::NOT_FOUND)?;
    let body = BodyStream(ReaderStream::with_capacity(file, 64 << 10));
    Response::builder()
        .body(body)
        .map_err(|_err| StatusCode::NOT_FOUND)
}

/// Opens the file with tokio, then uses a ReaderStream to read its contents to a single buffer.
///
/// Returns NOT_FOUND on any error.
async fn read_async(Path(relative_path): Path<String>) -> Result<impl IntoResponse, StatusCode> {
    let mut path = PathBuf::from("static");
    path.push(&relative_path);
    let file = tokio::fs::File::open(path)
        .await
        .map_err(|_err| StatusCode::NOT_FOUND)?;
    const BUF_LEN: usize = 64 << 10;
    let reader = ReaderStream::with_capacity(file, BUF_LEN);
    let results = reader
        .fold(BytesMut::with_capacity(BUF_LEN), |mut acc, b| {
            acc.extend(b.expect("error during stream"));
            acc
        })
        .await;
    Ok(results)
}

fn create_router() -> Router {
    Router::new()
        .route("/hello", get(hello_world))
        .nest_service("/serve_dir", ServeDir::new("static"))
        .route("/read/:path", get(read_file))
        .route("/stream/:path", get(stream_to_body))
        .route("/read_async/:path", get(read_async))
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
    test_static_files!(read, "/read");
    test_static_files!(stream, "/stream");
    test_static_files!(read_async, "/read_async");
}
