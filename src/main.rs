extern crate futures;
extern crate hyper;
extern crate pretty_env_logger;
extern crate tokio_core;
extern crate hyper_tls;
extern crate serde_json;
extern crate serde;

#[macro_use]
extern crate serde_derive;

use futures::Stream;
use futures::Future;

use self::hyper::{Body, Get, Post, StatusCode};
use self::hyper::header::ContentLength;
use self::hyper::server::{Http, Service, Request, Response};

pub mod reddit;

static NOTFOUND: &[u8] = b"Not Found";

struct ResponseExample(tokio_core::reactor::Handle);

impl Service for ResponseExample {
  type Request = Request;
  type Response = Response;
  type Error = hyper::Error;
  type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

  fn call(&self, req: Request) -> Self::Future {
    match (req.method(), req.path()) {
      (&Post, "/panic") => {
        reddit::get_top_aww_post(&self.0)
      }
      _ => {
        let body = Body::from("Not found");
        Box::new(futures::future::ok(Response::new()
                                     .with_status(StatusCode::NotFound)
                                     .with_header(ContentLength(NOTFOUND.len() as u64))
                                     .with_body(body)))
      }
    }
  }
}

fn main() {
  pretty_env_logger::init();

  let mut addr: std::net::SocketAddr = "0.0.0.0:3000".parse().unwrap();

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
