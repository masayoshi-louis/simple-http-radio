#[macro_use]
extern crate log;

use std::env;

use futures::future;
use futures::future::FutureResult;
use futures::prelude::*;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::service::service_fn;

mod audio;
mod flac;

const HTTP_STREAM_PATH: &'static str = "/stream.flac";

fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info,simple_http_radio=debug");
    }
    env_logger::init();

    audio::print_device_info();

    let addr = ([0, 0, 0, 0], 3000).into();

    let server = Server::bind(&addr)
        .serve(|| service_fn(handle))
        .map_err(|e| error!("server error: {}", e));

    info!("Listening on http://{}", addr);
    hyper::rt::run(server);
}

fn handle(req: Request<Body>) -> FutureResult<Response<Body>, hyper::Error> {
    let mut response = Response::new(Body::empty());
    let req_tuple = (req.method(), req.uri().path());
    {
        let (m, p) = req_tuple;
        debug!("Request: {} {}", m, p);
    }

    match req_tuple {
        (&Method::HEAD, HTTP_STREAM_PATH) => {
            set_headers(&mut response);
        }
        (&Method::GET, HTTP_STREAM_PATH) => {
            set_headers(&mut response);
            *response.body_mut() = Body::wrap_stream(audio::start());
        }
        (_, HTTP_STREAM_PATH) => {
            *response.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
        }
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    }

    return future::ok(response);
}

fn set_headers(response: &mut Response<Body>) {
    response.headers_mut().insert("Content-Type", "application/ogg".parse().unwrap());
    response.headers_mut().insert("Cache-Control", "no-cache, no-store".parse().unwrap());
}
