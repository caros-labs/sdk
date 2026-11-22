use caros_protocol::playback::*;
use tokio::io::{AsyncWriteExt, duplex};
#[tokio::test]
async fn stereo_pcm_survives_fragmentation() {
    let (mut sender, mut receiver) = duplex(7);
    let samples: Vec<i16> = (0..PLAYBACK_SAMPLES)
        .map(|i| if i % 2 == 0 { i16::MIN } else { i16::MAX })
        .collect();
    let expected = samples.clone();
    let task = tokio::spawn(async move {
        write_frame(&mut sender, &samples).await.unwrap();
    });
    assert_eq!(read_frame(&mut receiver).await.unwrap(), expected);
    task.await.unwrap();
}
#[tokio::test]
async fn rejects_wrong_size_and_truncated_pcm() {
    for bytes in [
        vec![0, 0, 0, 0],
        vec![255, 255, 255, 255],
        vec![0, 0, 15, 0, 1],
    ] {
        let (mut sender, mut receiver) = duplex(16);
        sender.write_all(&bytes).await.unwrap();
        drop(sender);
        assert!(read_frame(&mut receiver).await.is_err());
    }
}
