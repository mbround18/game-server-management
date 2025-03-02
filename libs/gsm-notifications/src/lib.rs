//! # gsm-notifications
//!
//! A generic notifications library that dispatches notifications to a webhook URL.
//! If the URL matches a Discord webhook pattern, it sends a Discord embed payload;
//! otherwise, it sends a generic JSON payload.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use gsm_notifications::{send_notification, NotificationError};
//!
//! // Send a generic notification (no extra data)
//! let webhook_url = "https://example.com/webhook";
//! send_notification(webhook_url, "INFO", "This is a generic message", Option::<()>::None)?;
//!
//! // Send a Discord notification (using embed formatting)
//! let discord_webhook = "https://discord.com/api/webhooks/1234567890/abcdef";
//! send_notification(discord_webhook, "ALERT", "Discord alert message", Option::<()>::None)?;
//! # Ok::<(), NotificationError>(())
//! ```

use reqwest::blocking::Client;
use serde::Serialize;
use std::error::Error;
use std::fmt;

/// Custom error type for notifications.
#[derive(Debug)]
pub enum NotificationError {
    HttpError(reqwest::Error),
    InvalidWebhookUrl(String),
    SerializationError(serde_json::Error),
    DispatcherNotFound(String),
}

impl fmt::Display for NotificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotificationError::HttpError(err) => write!(f, "HTTP error: {}", err),
            NotificationError::InvalidWebhookUrl(url) => write!(f, "Invalid webhook URL: {}", url),
            NotificationError::SerializationError(err) => write!(f, "Serialization error: {}", err),
            NotificationError::DispatcherNotFound(url) => {
                write!(f, "No dispatcher for webhook URL: {}", url)
            }
        }
    }
}

impl Error for NotificationError {}

impl From<reqwest::Error> for NotificationError {
    fn from(err: reqwest::Error) -> Self {
        NotificationError::HttpError(err)
    }
}

impl From<serde_json::Error> for NotificationError {
    fn from(err: serde_json::Error) -> Self {
        NotificationError::SerializationError(err)
    }
}

/// Generic payload for non–Discord notifications.
#[derive(Serialize)]
pub struct NotificationPayload<T: Serialize> {
    pub notification_type: String,
    pub message: String,
    pub data: Option<T>,
}

/// Checks that the webhook URL is non–empty and parses correctly.
fn validate_webhook_url(webhook_url: &str) -> Result<(), NotificationError> {
    if webhook_url.is_empty() || reqwest::Url::parse(webhook_url).is_err() {
        Err(NotificationError::InvalidWebhookUrl(
            webhook_url.to_string(),
        ))
    } else {
        Ok(())
    }
}

/// Returns true if the URL appears to be a Discord webhook.
#[allow(dead_code)]
fn is_discord_webhook(webhook_url: &str) -> bool {
    webhook_url.starts_with("https://discord.com/api/webhooks")
        || webhook_url.starts_with("https://discordapp.com/api/webhooks")
}

/// Discord embed structure.
#[derive(Serialize)]
struct DiscordEmbed {
    title: String,
    description: String,
    color: i32,
}

/// Discord webhook payload.
#[derive(Serialize)]
struct DiscordWebhookBody {
    content: String,
    embeds: Vec<DiscordEmbed>,
}

/// Returns a color value based on the notification type.
fn get_discord_color(notification_type: &str) -> i32 {
    match notification_type.to_lowercase().as_str() {
        "alert" => 0xFA113D,
        "info" => 0x4BB543,
        _ => 0x007F66,
    }
}

/// Object–safe trait for dispatching notifications. The method takes extra data
/// as an already–serialized JSON value.
pub trait NotificationDispatcher: Send + Sync {
    fn send_payload(
        &self,
        webhook_url: &str,
        notification_type: &str,
        message: &str,
        data: Option<serde_json::Value>,
    ) -> Result<(), NotificationError>;
}

/// Dispatcher for generic webhooks.
pub struct GenericDispatcher;

impl NotificationDispatcher for GenericDispatcher {
    fn send_payload(
        &self,
        webhook_url: &str,
        notification_type: &str,
        message: &str,
        data: Option<serde_json::Value>,
    ) -> Result<(), NotificationError> {
        let payload = NotificationPayload {
            notification_type: notification_type.to_string(),
            message: message.to_string(),
            data,
        };
        let client = Client::new();
        let response = client.post(webhook_url).json(&payload).send()?;
        response.error_for_status()?;
        Ok(())
    }
}

/// Dispatcher for Discord webhooks.
pub struct DiscordDispatcher;

impl NotificationDispatcher for DiscordDispatcher {
    fn send_payload(
        &self,
        webhook_url: &str,
        notification_type: &str,
        message: &str,
        _data: Option<serde_json::Value>, // Extra data is ignored for Discord.
    ) -> Result<(), NotificationError> {
        let payload = DiscordWebhookBody {
            content: format!("Notification: {}", notification_type),
            embeds: vec![DiscordEmbed {
                title: notification_type.to_string(),
                description: message.to_string(),
                color: get_discord_color(notification_type),
            }],
        };
        let client = Client::new();
        let response = client.post(webhook_url).json(&payload).send()?;
        response.error_for_status()?;
        Ok(())
    }
}

/// Type alias to simplify the complex type used in the dispatcher registry.
type DispatcherEntry = (
    Box<dyn Fn(&str) -> bool + Send + Sync>,
    Box<dyn NotificationDispatcher>,
);

/// A simple registry mapping a predicate (on the URL) to a dispatcher.
struct DispatcherRegistry {
    dispatchers: Vec<DispatcherEntry>,
}

impl DispatcherRegistry {
    fn new() -> Self {
        Self {
            dispatchers: Vec::new(),
        }
    }

    /// Registers a dispatcher with a predicate function.
    fn register<F>(&mut self, predicate: F, dispatcher: Box<dyn NotificationDispatcher>)
    where
        F: Fn(&str) -> bool + Send + Sync + 'static,
    {
        self.dispatchers.push((Box::new(predicate), dispatcher));
    }

    /// Finds the first dispatcher whose predicate returns true.
    fn get_dispatcher(&self, webhook_url: &str) -> Option<&DispatcherEntry> {
        self.dispatchers.iter().find(|(pred, _)| pred(webhook_url))
    }
}

/// Constructs a default dispatcher registry with Discord and generic dispatchers.
fn default_registry() -> DispatcherRegistry {
    let mut registry = DispatcherRegistry::new();
    registry.register(
        |url| {
            url.starts_with("https://discord.com/api/webhooks")
                || url.starts_with("https://discordapp.com/api/webhooks")
        },
        Box::new(DiscordDispatcher),
    );
    // Generic dispatcher as fallback.
    registry.register(|_url| true, Box::new(GenericDispatcher));
    registry
}

/// Sends a notification to the given webhook URL.
///
/// It converts any extra data into a JSON value and selects the appropriate dispatcher
/// based on the URL pattern.
///
/// # Parameters
/// - `webhook_url`: The target webhook URL.
/// - `notification_type`: A string label (e.g., "INFO", "ALERT").
/// - `message`: The notification message.
/// - `data`: Optional extra data (any serializable type).
///
/// # Returns
/// An Ok(()) on success, or a NotificationError.
pub fn send_notification<T: Serialize>(
    webhook_url: &str,
    notification_type: &str,
    message: &str,
    data: Option<T>,
) -> Result<(), NotificationError> {
    validate_webhook_url(webhook_url)?;
    let registry = default_registry();
    let data_value = match data {
        Some(d) => Some(serde_json::to_value(d)?),
        None => None,
    };
    if let Some((_, dispatcher)) = registry.get_dispatcher(webhook_url) {
        dispatcher.send_payload(webhook_url, notification_type, message, data_value)
    } else {
        Err(NotificationError::DispatcherNotFound(
            webhook_url.to_string(),
        ))
    }
}
