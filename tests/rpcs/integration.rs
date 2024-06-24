use protos::echo::v1::echo_service_client::EchoServiceClient;
use protos::echo::v1::UnaryEchoRequest;
use crate::setup_test_context;

#[tokio::test]
async fn echo() -> Result<(), Box<dyn std::error::Error>> {
    let (ctx, tx, jh) = setup_test_context("echo", 50200).await;
    let mut client = EchoServiceClient::connect(ctx.url.clone()).await.unwrap();

    let request = tonic::Request::new(UnaryEchoRequest {
        message: "hello".to_string(),
    });
    client.unary_echo(request).await?;

    tx.send(()).unwrap();
    jh.await.unwrap();
    ctx.cleanup().await;
    Ok(())
}

#[tokio::test]
async fn echo_invalid_message() -> Result<(), Box<dyn std::error::Error>> {
    let (ctx, tx, jh) = setup_test_context("echo_invalid_message", 50200).await;
    let mut client = EchoServiceClient::connect(ctx.url.clone()).await.unwrap();

    let request = tonic::Request::new(UnaryEchoRequest {
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
    ctx.cleanup().await;
    Ok(())
}
