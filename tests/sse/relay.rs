//! Redis pub/sub relay: an event published by one replica reaches the bus of every replica.

use std::time::Duration;

use mairie360_api_lib::test_setup::redis_setup::start_redis_container;
use message_api::sse::relay::RedisRelay;
use message_api::sse::state::{AppState, ChatEvent};
use serial_test::serial;
use testcontainers::core::{ContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::GenericImage;
use testcontainers::ImageExt;
use tokio::sync::broadcast;

/// Starts the relay of one replica and waits until it is subscribed.
async fn replica(url: &str, channel: &str) -> (AppState, broadcast::Receiver<ChatEvent>) {
    let relay = RedisRelay::with_channel(url, channel).expect("valid Redis URL");
    let (bus, rx) = broadcast::channel(16);
    tokio::spawn(relay.clone().run(bus.clone()));
    for _ in 0..100 {
        if relay.is_subscribed() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(relay.is_subscribed(), "the relay should subscribe");
    (AppState::new(bus, Some(relay)), rx)
}

async fn received(rx: &mut broadcast::Receiver<ChatEvent>) -> ChatEvent {
    tokio::time::timeout(Duration::from_secs(5), rx.recv())
        .await
        .expect("event relayed in time")
        .expect("bus open")
}

#[tokio::test]
#[serial]
async fn test_an_event_reaches_every_replica_once() {
    let (_redis, config) = start_redis_container().await;
    let (first, mut first_rx) = replica(&config.url, "sse:chat-events").await;
    let (_second, mut second_rx) = replica(&config.url, "sse:chat-events").await;

    let event = ChatEvent {
        chat_id: 5,
        sender_id: 42,
    };
    first.publish(event.clone()).await;

    assert_eq!(received(&mut first_rx).await, event);
    assert_eq!(received(&mut second_rx).await, event);
    // Relayed, not also sent locally: the publishing replica gets it once.
    assert!(
        tokio::time::timeout(Duration::from_millis(300), first_rx.recv())
            .await
            .is_err()
    );
}

#[tokio::test]
#[serial]
async fn test_an_unreachable_redis_falls_back_to_the_local_bus() {
    // Nothing listens on this port: the relay never subscribes and publishing fails.
    let relay = RedisRelay::with_channel("redis://127.0.0.1:1", "sse:chat-events").unwrap();
    let (bus, mut rx) = broadcast::channel(16);
    let state = AppState::new(bus, Some(relay));

    let event = ChatEvent {
        chat_id: 5,
        sender_id: 42,
    };
    state.publish(event.clone()).await;
    assert_eq!(received(&mut rx).await, event);
}

#[test]
fn test_the_channel_is_scoped_to_the_acl_role() {
    let relay = RedisRelay::new("redis://message-api:secret@redis:6379").unwrap();
    assert_eq!(relay.channel(), "message-api:sse:chat-events");
}

/// The ACL line the Deploiment chart writes for the `message-api` role: its keys, its own
/// channels, the cache-aside commands plus `PUBLISH` / `SUBSCRIBE`.
const MESSAGE_API_ACL: &str = "message-api on >acl-test-password ~message-api:* resetchannels \
     &message-api:* -@all +get +set +del +exists +expire +publish +subscribe";

#[tokio::test]
#[serial]
async fn test_the_relay_works_under_the_platform_acl() {
    let mut cmd = vec!["redis-server".to_string()];
    for line in [
        "default off resetkeys resetchannels -@all",
        "other-api on >acl-test-password ~other-api:* resetchannels -@all +get +set +del +exists +expire",
        MESSAGE_API_ACL,
    ] {
        cmd.push("--user".to_string());
        cmd.extend(line.split(' ').map(str::to_string));
    }
    let node = GenericImage::new("redis", "7.4-alpine")
        .with_exposed_port(ContainerPort::Tcp(6379))
        .with_wait_for(WaitFor::message_on_stdout("Ready to accept connections"))
        .with_cmd(cmd)
        .start()
        .await
        .expect("Redis with ACL");
    let host = node.get_host().await.unwrap();
    let port = node.get_host_port_ipv4(6379).await.unwrap();

    let url = format!("redis://message-api:acl-test-password@{host}:{port}");
    let relay = RedisRelay::new(&url).unwrap();
    let channel = relay.channel().to_string();
    let (state, mut rx) = replica(&url, &channel).await;
    let event = ChatEvent {
        chat_id: 7,
        sender_id: 1,
    };
    state.publish(event.clone()).await;
    assert_eq!(received(&mut rx).await, event);

    // Another role may not listen to the channel of message-api.
    let other = RedisRelay::with_channel(
        &format!("redis://other-api:acl-test-password@{host}:{port}"),
        &channel,
    )
    .unwrap();
    let (other_bus, _) = broadcast::channel(4);
    tokio::spawn(other.clone().run(other_bus));
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert!(!other.is_subscribed());
}
