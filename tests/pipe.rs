use async_pipe::Pipe;
use futures_util::StreamExt;
#[tokio::test]
async fn test_pipe() {
    let mut pipe = Pipe::new();

    pipe.write(1).await;

    let mut recv = pipe.clone();

    let next = recv.next().await;

    assert_eq!(next, Some(Some(1)));

    pipe.write(2).await;
    recv.write(3).await;

    let next = pipe.next().await;

    assert_eq!(next, Some(Some(3)));
}
