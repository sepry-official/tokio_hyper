use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use tokio::net::TcpListener;

use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt::format;
use std::net::SocketAddr;
use url::form_urlencoded;
use meval::eval_str;

static INDEX: &[u8] = b"<html><body><form action=\"post\" method=\"post\">Name: <input type=\"text\" name=\"name\"><br>Number: <input type=\"text\" name=\"number\"><br><input type=\"submit\"></body></html>";
static MISSING: &[u8] = b"Missing field";
static NOTNUMERIC: &[u8] = b"Number field is not numeric";



async fn param_example(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, Infallible>>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") | (&Method::GET, "/post") => Ok(Response::new(full(INDEX))),
        
        (&Method::GET, "/get") => {
            let query = if let Some(q) = req.uri().query() {
                q
            } else {
                return Ok(Response::builder()
                    .status(StatusCode::UNPROCESSABLE_ENTITY)
                    .body(full(MISSING))
                    .unwrap());
            };
            let params = form_urlencoded::parse(query.as_bytes())
                .into_owned()
                .collect::<HashMap<String, String>>();
            let default_param = String::from("1");
            let default_op = String::from("+");
            let bad_calculations = -999.999;


            let param_a = params.get("a").unwrap_or(&default_param);
            let param_b = params.get("b").unwrap_or(&default_param);
            let mut param_op = params.get("op").unwrap_or(&default_op);

            if param_op == " " {
                param_op = &default_op;
            }
            let calculations = match eval_str(format!("{} {} {}", param_a, param_op, param_b)){
                Ok(p) => p,
                Err(_) => bad_calculations
            };
            // let calculations = eval_str(format!("{} {} {}", param_a, param_op, param_b)).unwrap();
            let mut body = String::from("");
            if calculations != bad_calculations {
                body = format!("You requested {} {} {} = {}", param_a, param_op, param_b, calculations);
            } else {
                body = String::from("You have provided bad params and it is impossible to calculate");
            }
            // let body = format!("{} {} {}", param_a, param_op, param_b);
            
            Ok(Response::new(full(body)))
        }
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(empty())
            .unwrap()),
    }
}

fn empty() -> BoxBody<Bytes, Infallible> {
    Empty::<Bytes>::new().boxed()
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, Infallible> {
    Full::new(chunk.into()).boxed()
}



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8011));

    // We create a TcpListener and bind it to 127.0.0.1:3000
    let listener = TcpListener::bind(addr).await?;

    // We start a loop to continuously accept incoming connections
    loop {
        let (stream, _) = listener.accept().await?;

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            // Finally, we bind the incoming connection to our `hello` service
            if let Err(err) = http1::Builder::new()
                // `service_fn` converts our function in a `Service`
                .serve_connection(stream, service_fn(param_example))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}