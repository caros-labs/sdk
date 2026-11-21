use caros_protocol::microphone::{MICROPHONE_SAMPLES, MicrophonePacket};
use tokio::io::{AsyncWriteExt, duplex};

#[tokio::test]
async fn binary_pcm_preserves_signed_samples_and_stream_events() {
    let packets = vec![
        MicrophonePacket::Ready,
        MicrophonePacket::Frame {
            sequence: 42,
            timestamp_us: 100_000,
            samples: (0..MICROPHONE_SAMPLES)
                .map(|n| if n % 2 == 0 { i16::MIN } else { i16::MAX })
                .collect(),
        },
        MicrophonePacket::Gap { missed: 12 },
        MicrophonePacket::Unavailable,
    ];
    let (mut writer, mut reader) = duplex(8);
    let expected = packets.clone();
    let task = tokio::spawn(async move {
        for packet in packets {
            packet.write(&mut writer).await.unwrap();
        }
    });
    for packet in expected {
        assert_eq!(MicrophonePacket::read(&mut reader).await.unwrap(), packet);
    }
    task.await.unwrap();
}

#[tokio::test]
async fn invalid_sizes_tags_and_truncated_pcm_are_rejected() {
    for data in [
        vec![0, 0, 0, 0],
        vec![0, 0, 255, 255],
        vec![0, 0, 0, 1, 99],
        vec![0, 0, 0, 1, 0],
        vec![0, 0, 0, 9, 1, 0],
    ] {
        let (mut writer, mut reader) = duplex(64);
        writer.write_all(&data).await.unwrap();
        drop(writer);
        assert!(MicrophonePacket::read(&mut reader).await.is_err());
    }
}
