use std::path::PathBuf;

use iced::Subscription;
use iced::futures::Stream;
use iced::futures::channel::mpsc::Sender;
use iced::futures::future;

#[derive(Debug, Default, PartialEq)]
pub enum Dialog {
    #[default]
    Idle,
    Requested,
}

impl Dialog {
    pub fn request(&mut self) {
        *self = Self::Requested;
    }

    pub fn finish(&mut self) {
        *self = Self::Idle;
    }

    pub fn subscription(&self) -> Subscription<Option<PathBuf>> {
        if *self == Self::Requested {
            return Subscription::run(stream);
        }

        Subscription::none()
    }
}

fn stream() -> impl Stream<Item = Option<PathBuf>> {
    iced::stream::channel(1, |mut sender: Sender<Option<PathBuf>>| async move {
        std::thread::spawn(move || {
            let result = rfd::FileDialog::new()
                .add_filter("Markdown", &["md", "markdown"])
                .pick_file();
            let _ = sender.try_send(result);
        });

        future::pending::<()>().await
    })
}
