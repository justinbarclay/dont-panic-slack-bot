extern crate futures;
extern crate hyper;
extern crate pretty_env_logger;
extern crate tokio_core;
extern crate hyper_tls;

use futures::Stream;
use futures::Future;

use hyper::{Body, Chunk, Client, Get, Post, StatusCode};
use hyper_tls::HttpsConnector;
use hyper::error::Error;
use hyper::header::ContentLength;
use hyper::server::{Http, Service, Request, Response};

static NOTFOUND: &[u8] = b"Not Found";

pub type ResponseStream = Box<Stream<Item = Chunk, Error = Error>>;

struct ResponseExample(tokio_core::reactor::Handle);

impl Service for ResponseExample {
  type Request = Request;
  type Response = Response<ResponseStream>;
  type Error = hyper::Error;
  type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

  fn call(&self, req: Request) -> Self::Future {
    println!("made it into call");
    match (req.method(), req.path()) {
      (&Get, "/panic") => {
        let client = Client::configure()
          .connector(HttpsConnector::new(4, &self.0).unwrap())
          .build(&self.0);
        let mut req = Request::new(Get, "https://www.reddit.com/r/aww/top/.json?limit=1".parse().unwrap());
        let web_res_future = client.request(req);
        let mut length = 0;
        Box::new(web_res_future.map(move |web_res| {
          let body: ResponseStream = Box::new(web_res.body().map(move |b| {
            // let v = b.to_vec();
            // length = v.len();
            // String::from_utf8_lossy(&v).to_string()
            b
          }));
          Response::new()
            .with_status(StatusCode::Ok)
            .with_body(body)
        }))
      },
      _ => {
        let body: ResponseStream = Box::new(Body::from("Not found"));
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

  let addr = "127.0.0.1:3000".parse().unwrap();

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
