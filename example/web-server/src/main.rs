use hyper::rt::{self, Future};
use hyper::service::service_fn_ok;
use hyper::{Body, Request, Response, Server};

fn main() {
  let addr = ([127, 0, 0, 1], 8000).into();

  let server = Server::bind(&addr)
    .serve(|| {
      service_fn_ok(move |_: Request<Body>| {
        println!("got a request at {:?}", std::time::SystemTime::now());
        Response::new(Body::from("Hello World!"))
      })
    })
    .map_err(|e| eprintln!("server error: {}", e));

  println!("Listening on http://{}", addr);

  rt::run(server);
}
