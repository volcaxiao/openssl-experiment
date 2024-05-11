use openssl::ssl::{Ssl, SslAcceptor, SslContext, SslFiletype, SslMethod};
use tokio_openssl::SslStream;
use std::pin::Pin;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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
                    let mut tls_stream = SslStream::new(ssl, stream).unwrap();
                    SslStream::accept(Pin::new(&mut tls_stream)).await.unwrap();
                    // let stream = TokioIo::new(stream);
                    let stream = TokioIo::new(tls_stream);
                    let conn = http1::Builder::new().serve_connection(stream, service_fn(handle));
                    if let Err(err) = conn.await {
                        eprintln!("Error serving connection: {}", err);
                    }
                });
            }
            Err(err) => {
                eprintln!("Error accepting connection: {}", err)
            }
        };
    }
}

async fn handle(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))
}

