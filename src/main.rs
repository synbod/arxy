use std::collections::*;
use std::env::args;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use actix_web::client::Client;
use actix_web::{
    middleware, web, App, Error as actix_error, HttpRequest, HttpResponse, HttpServer,
};
use listenfd::ListenFd;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Forwardable {
    to: String,
    headers: Option<HashMap<String, String>>,
}

async fn forward(
    req: HttpRequest,
    body: web::Bytes,
    context: web::Data<HashMap<String, Forwardable>>,
    client: web::Data<Client>,
) -> Result<HttpResponse, actix_error> {
    let context = context.get_ref()[req.uri().path()].clone();
    let mut forward_to_url = Url::parse(context.to.as_str()).unwrap();
    forward_to_url.set_query(req.uri().query());

    // TODO: This forwarded implementation is incomplete as it only handles the inofficial
    // X-Forwarded-For header but not the official Forwarded one.
    let forwarded_req = client
        .request_from(forward_to_url.as_str(), req.head())
        .no_decompress();
    let forwarded_req = if let Some(addr) = req.head().peer_addr {
        forwarded_req.header("x-forwarded-for", format!("{}", addr.ip()))
    } else {
        forwarded_req
    };

    // add other headers
    let forwarded_req = if let Some(headers) = context.headers {
        if headers.len() > 0 {
            headers.keys().into_iter().fold(forwarded_req, |res, key| {
                res.header(key, headers[key].as_str())
            })
        } else {
            forwarded_req
        }
    } else {
        forwarded_req
    };

    let mut res = forwarded_req
        .send_body(body)
        .await
        .map_err(actix_error::from)?;

    let mut client_resp = HttpResponse::build(res.status());
    // Remove `Connection` as per
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Connection#Directives
    for (header_name, header_value) in res.headers().iter().filter(|(h, _)| *h != "connection") {
        client_resp.header(header_name.clone(), header_value.clone());
    }

    Ok(client_resp.body(res.body().await?))
}

fn read_forwardable_from_file<P: AsRef<Path>>(
    path: P,
) -> Result<HashMap<String, Forwardable>, Box<Error>> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let forwardables = serde_json::from_reader(reader)?;

    Ok(forwardables)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    // get context
    let forwardables = read_forwardable_from_file("arxy.config.json").unwrap();
    println!("{:#?}", forwardables);

    let mut server = HttpServer::new(move || {
        App::new()
            .data(Client::new())
            .data(forwardables.clone())
            .wrap(middleware::Logger::default())
            .default_service(web::post().to(forward))
    });

    let mut listenfd = ListenFd::from_env();

    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)?
    } else {
        let args: Vec<String> = args().collect();
        server.bind(format!(
            "127.0.0.1:{}",
            if args.len() > 1 {
                args[1].clone()
            } else {
                String::from("8080")
            }
        ))?
    };

    server.run().await
}
