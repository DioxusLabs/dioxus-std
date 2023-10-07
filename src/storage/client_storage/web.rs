use async_broadcast::broadcast;
use dioxus::prelude::*;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::OnceLock;
use uuid::Uuid;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use web_sys::{window, Storage};

use crate::storage::storage::{
    serde_to_string, try_serde_from_string,
    StorageBacking, StorageChannelPayload,
};
use crate::utils::channel::UseChannel;

fn local_storage() -> Option<Storage> {
    window()?.local_storage().ok()?
}

fn set<T: Serialize>(key: String, value: &T) {
    #[cfg(not(feature = "ssr"))]
    {
        let as_str = serde_to_string(value);
        local_storage().unwrap().set_item(&key, &as_str).unwrap();
    }
}

fn get<T: DeserializeOwned>(key: &str) -> Option<T> {
    #[cfg(not(feature = "ssr"))]
    {
        let s: String = local_storage()?.get_item(key).ok()??;
        try_serde_from_string(&s)
    }
    #[cfg(feature = "ssr")]
    None
}

#[derive(Clone)]
pub struct ClientStorage;

impl StorageBacking for ClientStorage {
    type Key = String;

    fn subscribe<T: DeserializeOwned + 'static>(
        cx: &ScopeState,
        key: &Self::Key,
    ) -> Option<UseChannel<StorageChannelPayload<Self>>> {
        let channel = CHANNEL.get_or_init(|| {
            let (tx, rx) = broadcast::<StorageChannelPayload<ClientStorage>>(5);
            let channel = UseChannel::new(Uuid::new_v4(), tx, rx.deactivate());
            let channel_clone = channel.clone();

            let closure = Closure::wrap(Box::new(move |e: web_sys::StorageEvent| {
                log::info!("Storage event: {:?}", e);
                let key: String = e.key().unwrap();
                let channel_clone_clone = channel_clone.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let result = channel_clone_clone
                        .send(StorageChannelPayload::<ClientStorage> { key })
                        .await;
                    match result {
                        Ok(_) => log::info!("Sent storage event"),
                        Err(err) => log::info!("Error sending storage event: {:?}", err),
                    }
                });
            }) as Box<dyn FnMut(web_sys::StorageEvent)>);
            window()
                .unwrap()
                .add_event_listener_with_callback("storage", closure.as_ref().unchecked_ref())
                .unwrap();
            closure.forget();
            channel
        });
        Some(channel.clone())
    }

    fn set<T: Serialize>(key: String, value: &T) {
        set(key, value);
    }

    fn get<T: DeserializeOwned>(key: &String) -> Option<T> {
        get(key)
    }
}

static CHANNEL: OnceLock<UseChannel<StorageChannelPayload<ClientStorage>>> = OnceLock::new();
