use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use iced::Subscription;
use iced::futures::channel::mpsc::Sender;
use iced::futures::future;
use iced::futures::stream::BoxStream;
use notify::{RecursiveMode, Watcher};

const DEBOUNCE: Duration = Duration::from_millis(100);

pub struct FileWatcher {
    path: Arc<PathBuf>,
}

impl FileWatcher {
    pub fn new(path: PathBuf) -> Self {
        let path = std::path::absolute(&path).unwrap_or(path);

        Self {
            path: Arc::new(path),
        }
    }

    pub fn subscription(&self) -> Subscription<()> {
        Subscription::run_with(Arc::clone(&self.path), stream)
    }
}

fn stream(path: &Arc<PathBuf>) -> BoxStream<'static, ()> {
    let path = path.as_ref().clone();

    Box::pin(iced::stream::channel(1, move |sender| async move {
        std::thread::spawn(move || watch(path, sender));
        future::pending().await
    }))
}

fn watch(path: PathBuf, mut sender: Sender<()>) {
    let Some(directory) = path.parent() else {
        return;
    };
    let (events, receiver) = std::sync::mpsc::channel();
    let Ok(mut watcher) = notify::recommended_watcher(move |event| {
        if let Ok(event) = event {
            let _ = events.send(event);
        }
    }) else {
        return;
    };

    if watcher
        .watch(directory, RecursiveMode::NonRecursive)
        .is_err()
    {
        return;
    }

    while let Ok(event) = receiver.recv() {
        if !touches(&event, &path) {
            continue;
        }

        std::thread::sleep(DEBOUNCE);

        while receiver.try_recv().is_ok() {}

        if sender.try_send(()).is_err() {
            return;
        }
    }
}

fn touches(event: &notify::Event, path: &Path) -> bool {
    let relevant = matches!(
        event.kind,
        notify::EventKind::Create(_) | notify::EventKind::Modify(_) | notify::EventKind::Remove(_)
    );

    relevant && event.paths.iter().any(|event_path| event_path == path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn editing_file_produces_debounced_event() {
        let directory = std::env::temp_dir().join(format!("mdrs-watch-{}", std::process::id()));
        let _ = fs::create_dir_all(&directory);
        let path = directory.join("document.md");
        fs::write(&path, "old").expect("fixture file");

        let (events, receiver) = std::sync::mpsc::channel();
        let mut watcher = notify::recommended_watcher(move |event| {
            if let Ok(event) = event {
                let _ = events.send(event);
            }
        })
        .expect("watcher");
        watcher
            .watch(&directory, RecursiveMode::NonRecursive)
            .expect("watch directory");

        fs::write(&path, "new").expect("write document");

        let mut touched = false;
        while let Ok(event) = receiver.recv_timeout(Duration::from_secs(2)) {
            if touches(&event, &path) {
                touched = true;
                break;
            }
        }

        assert!(touched);

        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn unrelated_file_does_not_touch_target() {
        let target = std::env::temp_dir().join("mdrs-target.md");
        let event = notify::Event::new(notify::EventKind::Modify(notify::event::ModifyKind::Any))
            .add_path(std::env::temp_dir().join("mdrs-other.md"));

        assert!(!touches(&event, &target));
    }

    #[test]
    fn access_event_does_not_touch_target() {
        let target = std::env::temp_dir().join("mdrs-target.md");
        let event = notify::Event::new(notify::EventKind::Access(notify::event::AccessKind::Any))
            .add_path(target.clone());

        assert!(!touches(&event, &target));
    }
}
