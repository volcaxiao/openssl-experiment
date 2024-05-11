use std::pin::Pin;
use std::sync::Arc;
use openssl::ssl::{SslAcceptor, SslContext, SslFiletype, SslMethod, Ssl};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_openssl::SslStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 加载 SSL 证书和私钥
	let mut ssl_ctx = SslContext::builder(SslMethod::tls()).unwrap();
	ssl_ctx.set_private_key_file("key.pem", SslFiletype::PEM).unwrap();
	ssl_ctx.set_certificate_file("cert.pem", SslFiletype::PEM).unwrap();
	ssl_ctx.check_private_key().unwrap();
	let ssl_ctx = ssl_ctx.build();
	// let ssl = Ssl::new(ssl_ctx.as_ref()).unwrap();

    let listener = TcpListener::bind("127.0.0.1:8443").await?;
	
	loop {
		match listener.accept().await {
			Ok((stream, _)) => {
				
				let ssl = Ssl::new(ssl_ctx.as_ref()).unwrap();
				tokio::spawn(async move {
					let mut stream = std::pin::pin!(SslStream::new(ssl, stream).unwrap());
					stream.as_mut().accept().await.unwrap();
					match stream.as_mut().do_handshake().await {
						Ok(_) => {
							if let Err(err) = process_connection(stream.as_mut()).await {
								eprintln!("Error processing connection: {}", err);
							}
						},
						Err(err) => {
						    eprintln!("Error do_handshake: {}", err);
						}
					}
					
				});
			},
			Err(err) => {
			    eprintln!("Error accepting connection: {}", err)
			}
		};
		
	}
}

async fn process_connection<'a>(mut stream: Pin<&mut tokio_openssl::SslStream<tokio::net::TcpStream>>) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = [0; 1024];
    
    loop {
        let n = stream.read(&mut buffer).await?;
        if n == 0 {
            return Ok(());
        }
        
        let response = "HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\nHello World!\n";

        stream.write_all(response.as_bytes()).await?;
		stream.flush().await?;
		stream.shutdown().await.unwrap();
    }
}