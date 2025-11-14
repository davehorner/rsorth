#[cfg(feature = "uses_iceoryx2")]
use sorth::runtime::built_ins::io_words::IoxSub;
#[cfg(feature = "uses_iceoryx2")]
use iceoryx2::prelude::*;

#[cfg(feature = "uses_iceoryx2")]
#[test]
fn test_iox_pubsub_send_recv() {
    // Create publisher and subscriber for the same service
    let service = "test_service";
    let node = NodeBuilder::new().create::<iceoryx2::service::ipc::Service>().unwrap();
    let pubsub = node
        .service_builder(&service.try_into().unwrap())
        .publish_subscribe::<[u8; 4096]>()
        .open_or_create()
        .unwrap();
    let mut publisher = pubsub.publisher_builder().create().unwrap();
    let mut subscriber = pubsub.subscriber_builder().create().unwrap();

    // Send a message
    let mut arr = [0u8; 4096];
    let msg = b"hello iceoryx2";
    arr[..msg.len()].copy_from_slice(msg);
    publisher.send_copy(arr).unwrap();

    // Receive the message
    let mut received = None;
    for _ in 0..10 {
        if let Ok(Some(sample)) = subscriber.receive() {
            let payload: &[u8; 4096] = sample.payload();
            let s = String::from_utf8_lossy(&payload[..msg.len()]);
            received = Some(s.to_string());
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert_eq!(received, Some("hello iceoryx2".to_string()));
}
