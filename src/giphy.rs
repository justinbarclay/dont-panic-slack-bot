extern crate futures;
extern crate hyper;
extern crate tokio_core;


use hyper::Client;
use tokio_core::reactor::Core;

fn things(term: String){
    let mut core = Core::new()?;

    let client = Client::new(&core.handle());

    let uri = "https://www.reddit.com/r/aww/top/.json?limit=1".parse()?;

    let work = client.get(uri).and_then(|res| {
        println!("Response: {}", res.status());

        res.body().for_each(|chunk| {
            io::stdout()
                .write_all(&chunk)
                .map_err(From::from)
        })
    });
    core.run(work)?;
}
