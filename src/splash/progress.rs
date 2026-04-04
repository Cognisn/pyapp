use std::sync::mpsc;

pub enum SplashMessage {
    UpdateStatus(String),
    UpdateProgress(f32),
    Close,
}

pub struct SplashHandle {
    sender: mpsc::Sender<SplashMessage>,
}

impl SplashHandle {
    pub fn new(sender: mpsc::Sender<SplashMessage>) -> SplashHandle {
        SplashHandle { sender }
    }

    pub fn update_status(&self, status: &str) {
        let _ = self.sender.send(SplashMessage::UpdateStatus(status.to_string()));
    }

    pub fn update_progress(&self, progress: f32) {
        let _ = self.sender.send(SplashMessage::UpdateProgress(progress.clamp(0.0, 1.0)));
    }

    pub fn close(&self) {
        let _ = self.sender.send(SplashMessage::Close);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_splash_message_channel() {
        let (tx, rx) = mpsc::channel();
        let handle = SplashHandle::new(tx);

        handle.update_status("Downloading Python...");
        handle.update_progress(0.5);
        handle.close();

        match rx.recv().unwrap() {
            SplashMessage::UpdateStatus(s) => assert_eq!(s, "Downloading Python..."),
            _ => panic!("expected UpdateStatus"),
        }
        match rx.recv().unwrap() {
            SplashMessage::UpdateProgress(p) => assert!((p - 0.5).abs() < f32::EPSILON),
            _ => panic!("expected UpdateProgress"),
        }
        match rx.recv().unwrap() {
            SplashMessage::Close => {}
            _ => panic!("expected Close"),
        }
    }

    #[test]
    fn test_progress_clamping() {
        let (tx, rx) = mpsc::channel();
        let handle = SplashHandle::new(tx);

        handle.update_progress(1.5);
        match rx.recv().unwrap() {
            SplashMessage::UpdateProgress(p) => assert!((p - 1.0).abs() < f32::EPSILON),
            _ => panic!("expected UpdateProgress"),
        }

        handle.update_progress(-0.5);
        match rx.recv().unwrap() {
            SplashMessage::UpdateProgress(p) => assert!(p.abs() < f32::EPSILON),
            _ => panic!("expected UpdateProgress"),
        }
    }
}
