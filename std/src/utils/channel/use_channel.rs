use async_broadcast::{broadcast, InactiveReceiver, Receiver, SendError, Sender, TrySendError};
use dioxus::prelude::ScopeState;
use uuid::Uuid;

/// Send and listen for messages between multiple components.
#[derive(Debug, Clone)]
pub struct UseChannel<MessageType: Clone> {
    id: Uuid,
    sender: Sender<MessageType>,
    inactive_receiver: InactiveReceiver<MessageType>,
}

impl<T: Clone> PartialEq for UseChannel<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<MessageType: Clone> UseChannel<MessageType> {
    /// Tries to send a message to all listeners of the channel.
    pub fn try_send(&self, msg: impl Into<MessageType>) -> Result<(), TrySendError<MessageType>> {
        self.sender.try_broadcast(msg.into()).map(|_| ())
    }

    /// Sends a message to all listeners of the channel.
    pub async fn send(&self, msg: impl Into<MessageType>) -> Result<(), SendError<MessageType>> {
        self.sender.broadcast(msg.into()).await.map(|_| ())
    }

    /// Create a receiver for the channel.
    /// You probably want to use [`super::use_listen_channel()`].
    pub fn receiver(&mut self) -> Receiver<MessageType> {
        self.inactive_receiver.clone().activate()
    }
}

/// Send and listen for messages between multiple components.
pub fn use_channel<MessageType: Clone + 'static>(
    cx: &ScopeState,
    size: usize,
) -> &UseChannel<MessageType> {
    cx.use_hook(|| {
        let id = Uuid::new_v4();
        let (sender, receiver) = broadcast::<MessageType>(size);
        UseChannel {
            id,
            sender,
            inactive_receiver: receiver.deactivate(),
        }
    })
}
