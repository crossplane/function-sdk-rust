//! Helpers shared by the server test binaries.

// Each test binary compiles this module and uses only some of it.
#![allow(dead_code)]

use function_sdk_rust::proto::v1::function_runner_service_client::FunctionRunnerServiceClient;

pub async fn http_get(port: u16, accept: Option<&str>) -> String {
    use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
    for _ in 0..50 {
        let Ok(mut conn) = tokio::net::TcpStream::connect(format!("127.0.0.1:{port}")).await else {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            continue;
        };
        let header = accept
            .map(|a| format!("Accept: {a}\r\n"))
            .unwrap_or_default();
        conn.write_all(format!("GET /metrics HTTP/1.1\r\nHost: x\r\n{header}\r\n").as_bytes())
            .await
            .expect("write request");
        let mut out = Vec::new();
        let _ = conn.read_to_end(&mut out).await;
        return String::from_utf8_lossy(&out).into_owned();
    }
    panic!("cannot connect to the metrics endpoint");
}

pub fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .expect("bind to an ephemeral port")
        .local_addr()
        .expect("read local addr")
        .port()
}

pub async fn connect_with_retry(
    port: u16,
) -> FunctionRunnerServiceClient<tonic::transport::Channel> {
    for _ in 0..50 {
        if let Ok(client) =
            FunctionRunnerServiceClient::connect(format!("http://127.0.0.1:{port}")).await
        {
            return client;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    panic!("cannot connect to the function under test");
}
