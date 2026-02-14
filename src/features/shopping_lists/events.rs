#[cfg(feature = "ssr")]
use std::collections::HashMap;
#[cfg(feature = "ssr")]
use std::sync::Arc;

#[cfg(feature = "ssr")]
use parking_lot::RwLock;
#[cfg(feature = "ssr")]
use tokio::sync::broadcast;

#[cfg(feature = "ssr")]
use super::models::ShoppingListEvent;

#[cfg(feature = "ssr")]
pub type EventBroadcaster = Arc<RwLock<HashMap<i64, broadcast::Sender<ShoppingListEvent>>>>;

#[cfg(feature = "ssr")]
pub fn create_broadcaster() -> EventBroadcaster {
    Arc::new(RwLock::new(HashMap::new()))
}

#[cfg(feature = "ssr")]
pub fn get_or_create_channel(
    broadcaster: &EventBroadcaster,
    list_id: i64,
) -> broadcast::Sender<ShoppingListEvent> {
    let mut map = broadcaster.write();
    map.entry(list_id)
        .or_insert_with(|| broadcast::channel(100).0)
        .clone()
}

#[cfg(feature = "ssr")]
pub fn broadcast_event(broadcaster: &EventBroadcaster, list_id: i64, event: ShoppingListEvent) {
    let map = broadcaster.read();
    if let Some(tx) = map.get(&list_id) {
        let _ = tx.send(event);
    }
}

#[cfg(feature = "ssr")]
pub fn cleanup_inactive_channels(broadcaster: &EventBroadcaster) {
    let mut map = broadcaster.write();
    map.retain(|_, tx| tx.receiver_count() > 0);
}
