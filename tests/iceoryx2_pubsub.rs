// This test only runs if built with --features uses_iceoryx2
#![cfg(feature = "uses_iceoryx2")]

use iceoryx2::prelude::*;

#[test]
fn test_iceoryx2_pubsub() {
    let node = NodeBuilder::new().create::<ipc::Service>().unwrap();
    let service = node
        .service_builder(&"TestService".try_into().unwrap())
        .publish_subscribe::<u64>()
        .open_or_create()
        .unwrap();

    let publisher = service.publisher_builder().create().unwrap();
    let subscriber = service.subscriber_builder().create().unwrap();

    publisher.send_copy(42u64).unwrap();
    let sample = subscriber.receive().unwrap().unwrap();
    assert_eq!(*sample.payload(), 42u64);
}