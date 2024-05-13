use http_body_util::Empty;
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::Request;
use hyper::Response;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use hyper_util::rt::TokioIo;
use hyper_tls::HttpsConnector;
use tokio::net::TcpStream;

#[allow(unused)]
async fn get(
    url: &hyper::Uri,
) -> Result<Response<Incoming>, Box<dyn std::error::Error + Send + Sync>> {
    let host = url.host().expect("[GET] url has no host");
    let port = url.port_u16().unwrap_or(80);
    let address = format!("{}:{}", host, port);
    let stream = TcpStream::connect(address).await?;
    let io = TokioIo::new(stream);
    let authority = url.authority().unwrap().clone();
    let client = Client::builder(TokioExecutor::new()).build::<_,Empty<Bytes>>(HttpsConnector::new());
    let res = client.get(url.clone()).await?;
    Ok(res)
}

#[allow(unused)]
async fn delete(
    url: &hyper::Uri,
) -> Result<Response<Incoming>, Box<dyn std::error::Error + Send + Sync>> {
    let host = url.host().expect("[DELETE] url has no host");
    let port = url.port_u16().unwrap_or(80);
    let address = format!("{}:{}", host, port);
    let stream = TcpStream::connect(address).await?;
    let io = TokioIo::new(stream);
    let authority = url.authority().unwrap().clone();
    let (mut sender, _conn) = hyper::client::conn::http1::handshake(io).await?;
    let req = Request::builder()
        .method("DELETE")
        .uri(url)
        .header(hyper::header::HOST, authority.as_str())
        .body(Empty::<Bytes>::new())?;

    let res = sender.send_request(req).await?;
    Ok(res)
}

#[allow(unused)]
async fn post(
    url: &hyper::Uri,
    body: Bytes,
) -> Result<Response<Incoming>, Box<dyn std::error::Error + Send + Sync>> {
    // 解析 URL 来获取主机名和端口
    let host = url.host().expect("[POST] url has no host");
    let port = url.port_u16().unwrap_or(80);
    let address = format!("{}:{}", host, port);

    // 连接到服务器
    let stream = TcpStream::connect(address).await?;
    let io = TokioIo::new(stream);

    // 获取授权部分，通常是主机名和端口
    let authority = url.authority().unwrap().clone();

    // 执行 HTTP/1.1 握手
    let (mut sender, _conn) = hyper::client::conn::http1::handshake(io).await?;
    // 构建 HTTP 请求
    let req = Request::builder()
        .method("POST")
        .uri(url)
        .header(hyper::header::HOST, authority.as_str())
        .header(hyper::header::CONTENT_TYPE, "application/json")
        .body(Full::new(body))?;

    // 发送请求并等待响应
    let res = sender.send_request(req).await?;

    Ok(res)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let res = get(&"https://baidu.com".parse().unwrap()).await?;
    println!("{:?}", res);
    Ok(())
}
