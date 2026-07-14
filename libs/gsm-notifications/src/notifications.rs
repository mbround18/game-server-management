use crate::{NotificationError, send_notification};
use gsm_shared::fetch_var;
use tracing::debug;

pub enum StandardServerEvents {
    PlayerJoined(String),
    PlayerLeft(String),
    Started,
    Stopping,
    Stopped,
}

/// Sends notifications based on the server event.
///
/// This function accepts a `Server` enum variant and sends a notification using the webhook URL defined in the
/// environment variable. If the webhook URL is missing, a debug message is logged and no notification is sent.
///
/// # Arguments
///
/// * `event` - A `Server` enum instance representing the server event.
///
/// # Returns
///
/// A `Result<(), NotificationError>` indicating success or failure of sending the notification.
///
/// # Errors
///
/// Returns any notification dispatch error produced by URL validation, serialization,
/// transport, or webhook response status checks.
pub fn send_notifications(event: StandardServerEvents) -> Result<(), NotificationError> {
    let server_name = fetch_var("NAME", "My Server");
    std::env::var("WEBHOOK_URL").map_or_else(
        |_| {
            debug!("Skipping notification, WEBHOOK_URL is not present.");
            Ok(())
        },
        |webhook_url| match event {
            StandardServerEvents::PlayerJoined(name) => send_notification::<Option<String>>(
                &webhook_url,
                &format!("{server_name}: Player Joined"),
                &format!("Player {name} has joined the adventure!"),
                None,
            ),
            StandardServerEvents::PlayerLeft(name) => send_notification::<Option<String>>(
                &webhook_url,
                &format!("{server_name}: Player Left"),
                &format!("Player {name} has left the adventure."),
                None,
            ),
            StandardServerEvents::Started => send_notification::<Option<String>>(
                &webhook_url,
                &format!("{server_name}: Server Started"),
                "The server has started successfully.",
                None,
            ),
            StandardServerEvents::Stopping => send_notification::<Option<String>>(
                &webhook_url,
                &format!("{server_name}: Server Stopping"),
                "The server is shutting down gracefully.",
                None,
            ),
            StandardServerEvents::Stopped => send_notification::<Option<String>>(
                &webhook_url,
                &format!("{server_name}: Server Stopped"),
                "The server has been stopped.",
                None,
            ),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn returns_ok_when_webhook_url_not_set() {
        let _guard = env_lock().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        unsafe { std::env::remove_var("WEBHOOK_URL") };

        assert!(send_notifications(StandardServerEvents::Started).is_ok());
        assert!(send_notifications(StandardServerEvents::Stopping).is_ok());
        assert!(send_notifications(StandardServerEvents::Stopped).is_ok());
        assert!(
            send_notifications(StandardServerEvents::PlayerJoined("Alice".to_owned())).is_ok()
        );
        assert!(send_notifications(StandardServerEvents::PlayerLeft("Alice".to_owned())).is_ok());
    }

    #[test]
    fn returns_err_when_webhook_url_is_invalid() {
        let _guard = env_lock().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        unsafe { std::env::set_var("WEBHOOK_URL", "not-a-url") };

        let result = send_notifications(StandardServerEvents::Started);
        assert!(result.is_err());

        unsafe { std::env::remove_var("WEBHOOK_URL") };
    }
}
