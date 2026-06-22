use crate::{NotificationError, send_notification};
use gsm_shared::fetch_var;
use tracing::debug;

/// Represents standard server lifecycle events that can trigger notifications.
pub enum StandardServerEvents {
    /// A player with the given name joined the server.
    PlayerJoined(String),
    /// A player with the given name left the server.
    PlayerLeft(String),
    /// The server has started successfully.
    Started,
    /// The server is in the process of shutting down.
    Stopping,
    /// The server has fully stopped.
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
pub fn send_notifications(event: StandardServerEvents) -> Result<(), NotificationError> {
    let server_name = fetch_var("NAME", "My Server");
    match std::env::var("WEBHOOK_URL") {
        Ok(webhook_url) => match event {
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
        Err(_) => {
            debug!("Skipping notification, WEBHOOK_URL is not present.");
            Ok(())
        }
    }
}
