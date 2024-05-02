use tokio::sync::oneshot;
use tonic::transport::Server;
use crate::tests::{TestContext};
use futures_util::FutureExt;
use protos::grpc::examples::echo::echo_client::EchoClient;
use protos::grpc::examples::echo::echo_server::EchoServer;
use protos::grpc::examples::echo::EchoRequest;

#[tokio::test]
async fn echo() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = TestContext::new("postgres://postgres:postgres@127.0.0.1", "echo", "redis://:@localhost:6380", 6061);
    let (tx, rx) = oneshot::channel();
    let service = ctx.service.clone();

    let jh = tokio::spawn(async move {
        Server::builder()
            .add_service(EchoServer::new(service))
            .serve_with_shutdown(ctx.addr, rx.map(|_| ()))
            .await
            .unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let mut client = EchoClient::connect(ctx.url.clone()).await.unwrap();

    let request = tonic::Request::new(EchoRequest {
        message: "hello".to_string(),
    });
    client.unary_echo(request).await?;

    tx.send(()).unwrap();
    jh.await.unwrap();
    Ok(())
}

#[tokio::test]
async fn echo_invalid_message() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = TestContext::new("postgres://postgres:postgres@127.0.0.1", "echo_invalid_message", "redis://:@localhost:6380", 6062);
    let (tx, rx) = oneshot::channel();
    let service = ctx.service.clone();

    let jh = tokio::spawn(async move {
        Server::builder()
            .add_service(EchoServer::new(service))
            .serve_with_shutdown(ctx.addr, rx.map(|_| ()))
            .await
            .unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let mut client = EchoClient::connect(ctx.url.clone()).await.unwrap();

    let request = tonic::Request::new(EchoRequest {
        message: "".to_string(),
    });

    match client.unary_echo(request).await {
        Ok(_) => panic!("expected error"),
        Err(e) => {
            assert_eq!(e.code(), tonic::Code::InvalidArgument);
        }
    }

    tx.send(()).unwrap();
    jh.await.unwrap();
    Ok(())
}
