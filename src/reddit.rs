
extern crate futures;
extern crate hyper;
extern crate tokio_core;
extern crate hyper_tls;
extern crate serde_json;
extern crate serde;

use futures::Stream;
use futures::Future;

use serde_json::Value;

use self::hyper::{Chunk, Client, Get, StatusCode};
use hyper_tls::HttpsConnector;
use self::hyper::error::Error as Error;
use self::hyper::server::{Request, Response};
use self::hyper::header::{Headers, ContentType};
#[derive(Serialize, Deserialize)]
struct Attachment {
  title: String,
  image_url: String,
}

#[derive(Serialize, Deserialize)]
struct SlackMessage {
  response_type: String,
  channel: String,
  attachments: [Attachment; 1],
}

fn parse_response(body: &Chunk) -> ::std::result::Result<String, Error> {
  let v: Value = serde_json::from_slice(&body).unwrap();
  let parsed_result = v["data"]["children"][0]["data"]["url"].to_string();
  if parsed_result.is_empty() || parsed_result == "null" {
    Err(hyper::error::Error::Status)
  } else {
    Ok(parsed_result.replace("\"", ""))
  }
}

fn make_slack_response(url: String) -> String {
  let attachment = Attachment {
    title: "Someone is panicing".to_string(),
    image_url: url.to_string(),
  };

  let message = SlackMessage {
    response_type: "ephemeral".to_string(),
    channel: "#general".to_string(),
    attachments: [attachment],
  };
  serde_json::to_string(&message).unwrap()
}

pub fn get_top_aww_post(
  handler: &tokio_core::reactor::Handle,
) -> Box<Future<Item = hyper::Response, Error = hyper::Error>> {
  let client = Client::configure()
    .connector(HttpsConnector::new(4, handler).unwrap())
    .build(handler);
  let req = Request::new(
    Get,
    "https://www.reddit.com/r/aww/top/.json?limit=1"
      .parse()
      .unwrap(),
  );
  let web_res_future = client.request(req);

  Box::new(web_res_future.and_then(|web_res| {
    web_res.body().concat2().and_then(move |body| {
      let slack_message = match parse_response(&body) {
        Ok(response) => {
          make_slack_response(response)
        },
        Err(_e) => "Error parsing JSON".to_string(),
      };
      println!("{:?}", slack_message);
      let mut headers = Headers::new();
      headers.set(
        ContentType::json()
      );
      Ok(
        Response::new()
          .with_headers(headers)
          .with_status(StatusCode::Ok)
          .with_body(slack_message),
      )
    })
  }))
}
