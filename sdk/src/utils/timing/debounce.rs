use dioxus::prelude::*;
use futures::{
    channel::mpsc::{self, UnboundedSender as Sender},
    StreamExt,
};
use std::time::Duration;

/// The interface for calling a debounce.
///
/// See [`use_debounce`] for more information.
#[derive(PartialEq)]
pub struct UseDebounce<T: 'static> {
    sender: Signal<Sender<T>>,
}

impl<T> UseDebounce<T> {
    /// Will start the debounce countdown, resetting it if already started.
    pub fn action(&mut self, data: T) {
        self.sender.write().unbounded_send(data).ok();
    }
}

impl<T> Clone for UseDebounce<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for UseDebounce<T> {}

/// A hook for allowing a function to be called only after a provided [`Duration`] has passed.
///
/// Once the [`UseDebounce::action`] method is called, a timer will start counting down until
/// the callback is ran. If the [`UseDebounce::action`] method is called again, the timer will restart.
///
/// # Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_sdk::utils::timing::use_debounce;
/// use std::time::Duration;
///
/// fn App() -> Element {
///     let mut debounce = use_debounce(Duration::from_millis(2000), |_| println!("ran"));
///     
///     rsx! {
///         button {
///             onclick: move |_| {
///                 debounce.action(());
///             },
///             "Click!"
///         }
///     }
/// }
/// ```
pub fn use_debounce<T>(time: Duration, cb: impl FnOnce(T) + Copy + 'static) -> UseDebounce<T> {
    use_hook(|| {
        let (sender, mut receiver) = mpsc::unbounded();
        let debouncer = UseDebounce {
            sender: Signal::new(sender),
        };

        spawn(async move {
            let mut current_task: Option<Task> = None;

            loop {
                if let Some(data) = receiver.next().await {
                    if let Some(task) = current_task.take() {
                        task.cancel();
                    }

                    current_task = Some(spawn(async move {
                        tokio::time::sleep(time).await;
                        cb(data);
                    }));
                }
            }
        });

        debouncer
    })
}
