extern crate notify;

use self::notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher};
use error::ParcelWatcherError;
use std::path::PathBuf;
use std::result::Result;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;
use std::vec::Vec;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
const DEBOUNCE_DELAY: Duration = Duration::from_millis(10);
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
const DEBOUNCE_DELAY: Duration = Duration::from_millis(1000);

#[derive(Clone, Debug)]
pub struct ParcelWatcherEvent {
    event_type: String,
    path: String,
}

type WatcherCallback = fn(ParcelWatcherEvent) -> ();

pub struct ParcelWatcher {
    notify_receiver: Receiver<DebouncedEvent>,
    notify_watcher: RecommendedWatcher,
    callbacks: Vec<WatcherCallback>,
}

impl ParcelWatcher {
    pub fn new() -> Result<ParcelWatcher, ParcelWatcherError> {
        // Create a channel to receive the events.
        let (tx, rx) = channel();
        let notify_watcher = NotifyWatcher::new(tx, DEBOUNCE_DELAY)?;
        let parcel_watcher_instance = ParcelWatcher {
            notify_receiver: rx,
            notify_watcher,
            callbacks: Vec::new(),
        };
        return Ok(parcel_watcher_instance);
    }

    fn broadcast_event(&self, event_type: &str, path: PathBuf) {
        match path.to_str() {
            Some(path_str) => {
                let event = ParcelWatcherEvent {
                    event_type: String::from(event_type),
                    path: String::from(path_str),
                };

                println!("{:?}", event);

                for callback in self.callbacks.iter() {
                    callback(event.clone());
                }
            }
            None => {}
        }
    }

    pub fn process_events(&self) {
        // This is a simple loop, but you may want to use more complex logic here,
        // for example to handle I/O.
        for event in self.notify_receiver.try_iter() {
            println!("Got event: {:?}", event);

            match event {
                notify::DebouncedEvent::Create(p) => self.broadcast_event("create", p),
                notify::DebouncedEvent::Remove(p) => self.broadcast_event("delete", p),
                notify::DebouncedEvent::Write(p) => self.broadcast_event("update", p),
                notify::DebouncedEvent::Chmod(p) => self.broadcast_event("update", p),
                notify::DebouncedEvent::Rename(p1, p2) => {
                    self.broadcast_event("delete", p1);
                    self.broadcast_event("create", p2);
                }
                // TODO: NoticeWrite and NoticeRemove?
                // Default case, we don't care about a lot of events...
                _ => {}
            }
        }
    }

    pub fn watch(&mut self, directory_to_watch: &str) -> Result<(), ParcelWatcherError> {
        self.notify_watcher
            .watch(directory_to_watch, RecursiveMode::Recursive)?;

        return Ok(());
    }
}
