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
// 异步函数，用于获取URL
async fn get(
    url: &hyper::Uri,
) -> Result<Response<Incoming>, Box<dyn std::error::Error + Send + Sync>> {
    // 从URL中获取主机名
    let host = url.host().expect("[GET] url has no host");
    // 从URL中获取端口号，如果没有则默认为80
    let port = url.port_u16().unwrap_or(80);
    // 拼接主机名和端口号
    let address = format!("{}:{}", host, port);
    // 建立与主机的TCP连接
    let stream = TcpStream::connect(address).await?;
    // 将TCP流转换为TokioIo
    let io = TokioIo::new(stream);
    // 从URL中获取作者名称
    let authority = url.authority().unwrap().clone();
    // 构建客户端
    let client = Client::builder(TokioExecutor::new()).build::<_,Empty<Bytes>>(HttpsConnector::new());
    // 发起GET请求
    let res = client.get(url.clone()).await?;
    // 返回结果
    Ok(res)
}

#[allow(unused)]
// 异步函数，用于删除指定url
async fn delete(
    url: &hyper::Uri,
) -> Result<Response<Incoming>, Box<dyn std::error::Error + Send + Sync>> {
    // 从url中获取主机和端口
    let host = url.host().expect("[DELETE] url has no host");
    let port = url.port_u16().unwrap_or(80);
    let address = format!("{}:{}", host, port);
    // 使用TcpStream连接主机和端口
    let stream = TcpStream::connect(address).await?;
    let io = TokioIo::new(stream);
    // 从url中获取 authority
    let authority = url.authority().unwrap().clone();
    // 进行HTTP1的握手
    let (mut sender, _conn) = hyper::client::conn::http1::handshake(io).await?;
    // 创建一个DELETE请求
    let req = Request::builder()
        .method("DELETE")
        .uri(url)
        .header(hyper::header::HOST, authority.as_str())
        .body(Empty::<Bytes>::new())?;

    // 发送请求
    let res = sender.send_request(req).await?;
    // 返回结果
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
