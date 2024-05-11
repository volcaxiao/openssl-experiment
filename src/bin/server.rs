use openssl::ssl::{Ssl, SslAcceptor, SslContext, SslFiletype, SslMethod};
use tokio_openssl::SslStream;
use std::io::{BufReader, Read};
use std::pin::Pin;
use std::sync::Arc;

use std::convert::Infallible;
use std::net::SocketAddr;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::upgrade;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

// async fn hello(_: Request<impl hyper::body::Body>) -> Result<Response<Bytes>> {
//     Ok(Response::new(Bytes::from("Hello World!")))
// }
const ROOT_PATH: &str = "./SSL/WebServer";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 加载 SSL 证书和私钥
    let mut acceptor = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    acceptor
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .unwrap();
    acceptor.set_certificate_chain_file("cert.pem").unwrap();
    acceptor.check_private_key().unwrap();
    let acceptor = Arc::new(acceptor.build());
    // let ssl = Ssl::new(ssl_ctx.as_ref()).unwrap();

    let listener = TcpListener::bind("127.0.0.1:8443").await?;

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let acceptor = acceptor.clone();
                tokio::spawn(async move {
                    let ssl = Ssl::new(acceptor.context()).unwrap();

                    let mut tls_stream = match SslStream::new(ssl, stream) {
                        Ok(stream) => stream,
                        Err(err) => {
                            eprintln!("Error creating SSL stream: {}", err);
                            return ;
                        }
                    };

                    if let Err(error) = SslStream::accept(Pin::new(&mut tls_stream)).await {
                        eprintln!("Error accepting connection: {}", error);
                        return ;
                    }
                    // let stream = TokioIo::new(stream);
                    let stream = TokioIo::new(tls_stream);
                    let conn = http1::Builder::new().serve_connection(stream, service_fn(handle));
                    if let Err(err) = conn.await {
                        eprintln!("Error serving connection: {}", err);
                        return ;
                    }
                });
            }
            Err(err) => {
                eprintln!("Error accepting connection: {}", err)
            }
        };
    }
}

async fn handle(request: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    match *request.method() {
        hyper::Method::GET => {
            let path = request.uri().path();

            let file_path = format!("{}/{}", ROOT_PATH, path);
            let mut file = match std::fs::File::open(file_path) {
                Ok(file) => file,
                Err(_) => return Ok(Response::builder()
                    .status(hyper::StatusCode::NOT_FOUND)
                    .body(Full::new(Bytes::from("404 Not Found")))
                    .unwrap()),
            };
            let mut buf = String::new();
            let buf = match file.read_to_string(&mut buf) {
                Ok(_) => buf,
                Err(_) => return Ok(Response::builder()
                    .status(hyper::StatusCode::NOT_FOUND)
                    .body(Full::new(Bytes::from("404 Not Found")))
                    .unwrap()),
            };
            let response = Response::new(Full::new(Bytes::from(buf)));
            Ok(response)
        },
        _ => {
            let response = Response::builder()
                .status(hyper::StatusCode::NOT_FOUND)
                .body(Full::new(Bytes::from("404 Not Found")))
                .unwrap();
            Ok(response)
        }
    }
}



