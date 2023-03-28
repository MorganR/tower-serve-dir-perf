use std::{
    error::Error,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    pin::Pin,
    task::{Context, Poll, Waker},
    thread,
    time::Duration,
};

use axum::{
    body::Bytes,
    extract::Path,
    response::{IntoResponse, Response},
    routing::get,
    Router, Server,
};
use http_body::Body;
use hyper::{HeaderMap, StatusCode};
use tokio::io;

/// Writes a number of lines of output all at once.
async fn single(Path(n): Path<usize>) -> Result<impl IntoResponse, StatusCode> {
    let mut response: Vec<String> = Vec::with_capacity(n);
    for i in 1..=n {
        response.push(format!("Hello {}", i));
    }
    Ok(response.join("\n"))
}

/// A streaming body response, that sends a number of lines of text.
struct SimpleBody {
    n: usize,
    current: usize,
    is_pending: bool,
}

impl SimpleBody {
    fn new(n: usize) -> Self {
        Self {
            n,
            current: 0,
            is_pending: true,
        }
    }
}

impl Body for SimpleBody {
    type Data = Bytes;
    type Error = io::Error;

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        if self.current == self.n {
            return Poll::Ready(None);
        }
        if !self.is_pending {
            let with_new_line = self.current < self.n - 1;
            let self_mut = self.get_mut();
            self_mut.is_pending = true;
            self_mut.current += 1;
            return Poll::Ready(Some(Ok(Bytes::from(match with_new_line {
                true => format!("Hello {}\n", self_mut.current),
                false => format!("Hello {}", self_mut.current),
            }))));
        }

        self.get_mut().is_pending = false;
        cx.waker().wake_by_ref();
        Poll::Pending
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(None))
    }
}

/// A streaming body response, that sends a number of lines of text, with a 1ms pause between each.
struct DelayedBody {
    n: usize,
    current: usize,
    waker: Option<Waker>,
}

impl DelayedBody {
    fn new(n: usize) -> Self {
        Self {
            n,
            current: 0,
            waker: None,
        }
    }
}

impl Body for DelayedBody {
    type Data = Bytes;
    type Error = io::Error;

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        if self.current == self.n {
            return Poll::Ready(None);
        }
        if let Some(_) = self.waker {
            let with_new_line = self.current < self.n - 1;
            let self_mut = self.get_mut();
            self_mut.waker = None;
            self_mut.current += 1;
            return Poll::Ready(Some(Ok(Bytes::from(match with_new_line {
                true => format!("Hello {}\n", self_mut.current),
                false => format!("Hello {}", self_mut.current),
            }))));
        }

        let waker = cx.waker().clone();
        self.get_mut().waker = Some(waker.clone());

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(1));
            waker.wake();
        });
        Poll::Pending
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(None))
    }
}

/// Writes a number of lines of output via polling a Body.
async fn body(Path(n): Path<usize>) -> Result<impl IntoResponse, StatusCode> {
    let simple_body = SimpleBody::new(n);
    Response::builder()
        .body(simple_body)
        .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Writes a number of lines of output via polling a delayed body.
async fn delayed_body(Path(n): Path<usize>) -> Result<impl IntoResponse, StatusCode> {
    let body = DelayedBody::new(n);
    Response::builder()
        .body(body)
        .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)
}

fn create_router() -> Router {
    Router::new()
        .route("/single/:n", get(single))
        .route("/body/:n", get(body))
        .route("/delayed_body/:n", get(delayed_body))
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

    macro_rules! test_lines {
        ($prefix:ident, $base_path:expr) => {
            paste! {
            #[tokio::test]
                async fn [<$prefix _zero_lines>]() -> Result<(), Box<dyn Error>> {
                    let server = TestServer::new(create_router().into_make_service()).unwrap();

                    let path = format!("{}/0", $base_path);
                    let response = server.get(&path).await;

                    assert_eq!(response.text(), "");
                    Ok(())
                }
            }

            paste! {
                #[tokio::test]
                async fn [<$prefix _some_lines>]() -> Result<(), Box<dyn Error>> {
                    let server = TestServer::new(create_router().into_make_service()).unwrap();

                    let path = format!("{}/3", $base_path);
                    let response = server.get(&path).await;

                    assert_eq!(response.text(), "Hello 1\nHello 2\nHello 3");
                    Ok(())
                }
            }
        };
    }

    test_lines!(single, "/single");
    test_lines!(body, "/body");
    test_lines!(delayed_body, "/delayed_body");
}
