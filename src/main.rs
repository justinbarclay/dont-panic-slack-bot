extern crate futures;
extern crate hyper;
extern crate pretty_env_logger;
extern crate tokio_core;
extern crate hyper_tls;
extern crate serde_json;

use futures::Stream;
use futures::Future;

use serde_json::Value;

use hyper::{Body, Chunk, Client, Get, Post, StatusCode};
use hyper_tls::HttpsConnector;
use hyper::error::Error;
use hyper::header::ContentLength;
use hyper::server::{Http, Service, Request, Response};

static NOTFOUND: &[u8] = b"Not Found";

//pub type ResponseStream = Bkox<Stream<Item = Chunk, Error = Error>>;

struct ResponseExample(tokio_core::reactor::Handle);

impl Service for ResponseExample {
  type Request = Request;
  type Response = Response;
  type Error = hyper::Error;
  type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

  fn call(&self, req: Request) -> Self::Future {
    let client = Client::configure()
      .connector(HttpsConnector::new(4, &self.0).unwrap())
      .build(&self.0);

    match (req.method(), req.path()) {
      (&Get, "/panic") => {

        let mut req = Request::new(Get, "https://www.reddit.com/r/aww/top/.json?limit=1".parse().unwrap());
        let web_res_future = client.request(req);

        Box::new(web_res_future.and_then(|web_res| {

          web_res.body().concat2().and_then( move |body| {
            let v: Value = serde_json::from_slice(&body).unwrap();
            let url = v["data"]["children"][0]["data"]["url"].to_string();
            Box::new(
              futures::future::ok(
                Response::new()
                  .with_status(StatusCode::Ok)
                  .with_body(url))
            )
          })
            .map(|x| {
              x
            })
        }).map(|x| {
          x
        }))
      }
      _ => {
        let body = Body::from("Not found");
        Box::new(futures::future::ok(Response::new()
                                     .with_status(StatusCode::NotFound)
                                     .with_header(ContentLength(NOTFOUND.len() as u64))
                                     .with_body("Not found")))
      }
    }
  }
}

fn main() {
  pretty_env_logger::init();

  let mut addr: std::net::SocketAddr = "127.0.0.1:3000".parse().unwrap();
  //  let port: i32 = std::env::args("RUST_PORT").parse().unwrap();

  let port = match std::env::var("RUST_PORT") {
    Ok(val) => val.parse::<u16>().unwrap(),
    Err(_) => "3000".parse::<u16>().unwrap()
  };

  addr.set_port(port);
  let mut core = tokio_core::reactor::Core::new().unwrap();
  let server_handle = core.handle();
  let client_handle = core.handle();

  let serve = Http::new().serve_addr_handle(&addr, &server_handle, move || Ok(ResponseExample(client_handle.clone()))).unwrap();
  println!("Listening on http://{} with 1 thread.", serve.incoming_ref().local_addr());


   let h2 = server_handle.clone();
    server_handle.spawn(serve.for_each(move |conn| {
        h2.spawn(conn.map(|_| ()).map_err(|err| println!("serve error: {:?}", err)));
        Ok(())
    }).map_err(|_| ()));
  core.run(futures::future::empty::<(), ()>()).unwrap();
}
