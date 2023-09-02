use async_pipe::Pipe;
use futures_util::StreamExt;
#[tokio::test]
async fn test_pipe() {
    let mut pipe = Pipe::new();

    pipe.send(1);

    let mut recv = pipe.clone();

    let next = recv.next().await;

    assert_eq!(next, Some(1));

    pipe.send(2);
    recv.send(3);

    let next = pipe.next().await;

    assert_eq!(next, Some(3));

    pipe.finish();

    let next = recv.next().await;

    assert_eq!(next, None);
}
