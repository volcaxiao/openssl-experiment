use hyper::{Body, Client, Method, Request, Response};
use hyper::client::HttpConnector;
use tokio;

async fn send_request(client: &Client<HttpConnector>, method: Method, uri: hyper::Uri, body: Option<Body>) -> Result<Response<Body>, hyper::Error> {
    let req_builder = Request::builder()
        .method(method)
        .uri(uri);

    let req = match body {
        Some(b) => req_builder.body(b),
        None => req_builder.body(Body::empty()),
    }?;

    client.request(req).await
}

async fn get(client: &Client<HttpConnector>, url: &str) -> Result<String, hyper::Error> {
    let uri = url.parse().unwrap();
    let res = send_request(client, Method::GET, uri, None).await?;
    let body_bytes = hyper::body::to_bytes(res.into_body()).await?;
    Ok(String::from_utf8_lossy(&body_bytes).to_string())
}

async fn post(client: &Client<HttpConnector>, url: &str, body_data: &str) -> Result<String, hyper::Error> {
    let uri = url.parse().unwrap();
    let res = send_request(client, Method::POST, uri, Some(Body::from(body_data.to_string()))).await?;
    let body_bytes = hyper::body::to_bytes(res.into_body()).await?;
    Ok(String::from_utf8_lossy(&body_bytes).to_string())
}

async fn delete(client: &Client<HttpConnector>, url: &str) -> Result<String, hyper::Error> {
    let uri = url.parse().unwrap();
    let res = send_request(client, Method::DELETE, uri, None).await?;
    let body_bytes = hyper::body::to_bytes(res.into_body()).await?;
    Ok(String::from_utf8_lossy(&body_bytes).to_string())
}

#[tokio::main]
async fn main() {
    // let client = Client::new();

    // // 使用GET请求
    // if let Ok(res) = get(&client, "http://127.0.0.1/").await {
    //     println!("GET response: {}", res);
    // }
}