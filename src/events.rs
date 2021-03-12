use crate::client::{Client, Error};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertType {
    Error,
    Warning,
    Info,
    Success,
    UserUpdate,
    Recommendation,
    Snapshot,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Normal,
    Low,
}

/// # Examples
///
/// ```
/// use datadog_client::events::{CreateEventPayload, AlertType, Priority};
///
/// let event = CreateEventPayload::new(
///     "Some Title".to_string(),
///     "Some Text in Markdown".to_string(),
/// )
///     .set_aggregation_key("whatever".to_string())
///     .add_tag("environment:prod".to_string());
/// ```
#[derive(Debug, Serialize)]
pub struct CreateEventPayload {
    // An arbitrary string to use for aggregation. Limited to 100 characters. If you specify
    // a key, all events using that key are grouped together in the Event Stream.
    #[serde(skip_serializing_if = "Option::is_none")]
    aggregation_key: Option<String>,
    // If an alert event is enabled, set its type. For example, error, warning, info, success,
    // user_update, recommendation, and snapshot.
    #[serde(skip_serializing_if = "Option::is_none")]
    alert_type: Option<AlertType>,
    // POSIX timestamp of the event. Must be sent as an integer (i.e. no quotes).
    // Limited to events no older than 7 days.
    #[serde(skip_serializing_if = "Option::is_none")]
    date_happened: Option<i64>,
    // A device name
    #[serde(skip_serializing_if = "Option::is_none")]
    device_name: Option<String>,
    // Host name to associate with the event. Any tags associated with the host are also applied to this event.
    #[serde(skip_serializing_if = "Option::is_none")]
    host: Option<String>,
    // The priority of the event. For example, normal or low.
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<Priority>,
    // ID of the parent event. Must be sent as an integer (i.e. no quotes).
    #[serde(skip_serializing_if = "Option::is_none")]
    related_event_id: Option<i64>,
    // The type of event being posted.
    // Option examples include nagios, hudson, jenkins, my_apps, chef, puppet, git, bitbucket, etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    source_type_name: Option<String>,
    // A list of tags to apply to the event.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
    // The body of the event. Limited to 4000 characters. The text supports markdown.
    // To use markdown in the event text, start the text block with %%% \n and end the text block with \n %%%.
    // Use msg_text with the Datadog Ruby library.
    text: String,
    // The event title. Limited to 100 characters.
    title: String,
}

impl CreateEventPayload {
    pub fn new(title: String, text: String) -> Self {
        Self {
            aggregation_key: None,
            alert_type: None,
            date_happened: None,
            device_name: None,
            host: None,
            priority: None,
            related_event_id: None,
            source_type_name: None,
            tags: Vec::new(),
            title,
            text,
        }
    }

    pub fn set_aggregation_key(mut self, value: String) -> Self {
        self.aggregation_key = Some(value);
        self
    }

    pub fn set_alert_type(mut self, value: AlertType) -> Self {
        self.alert_type = Some(value);
        self
    }

    pub fn set_date_happened(mut self, value: i64) -> Self {
        self.date_happened = Some(value);
        self
    }

    pub fn set_device_name(mut self, value: String) -> Self {
        self.device_name = Some(value);
        self
    }

    pub fn set_host(mut self, value: String) -> Self {
        self.host = Some(value);
        self
    }

    pub fn set_priority(mut self, value: Priority) -> Self {
        self.priority = Some(value);
        self
    }

    pub fn set_related_event_id(mut self, value: i64) -> Self {
        self.related_event_id = Some(value);
        self
    }

    pub fn set_source_type_name(mut self, value: String) -> Self {
        self.source_type_name = Some(value);
        self
    }

    pub fn add_tag(mut self, value: String) -> Self {
        self.tags.push(value);
        self
    }

    pub fn set_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

impl Client {
    /// Post an event
    ///
    /// This endpoint allows you to post events to the stream.
    /// Tag them, set priority and event aggregate them with other events.
    ///
    /// https://docs.datadoghq.com/api/latest/events/#post-an-event
    ///
    pub async fn post_event(&self, event: &CreateEventPayload) -> Result<(), Error> {
        self.post("/api/v1/events", event).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::Config;
    use mockito::mock;

    #[tokio::test]
    async fn post_metrics_success() {
        let call = mock("POST", "/api/v1/events").with_status(202).create();
        let client = Client::new(Config::new(
            mockito::server_url(),
            String::from("fake-api-key"),
        ));
        let event = CreateEventPayload::new(
            String::from("Some Event Title"),
            String::from("Some event text"),
        )
        .add_tag(String::from("testing"));
        let result = client.post_event(&event).await;
        assert!(result.is_ok());
        call.expect(1);
    }

    #[tokio::test]
    async fn post_metrics_unauthorized() {
        let call = mock("POST", "/api/v1/series")
            .with_status(403)
            .with_body("{\"errors\":[\"Authentication error\"]}")
            .create();
        let client = Client::new(Config::new(
            mockito::server_url(),
            String::from("fake-api-key"),
        ));
        let event = CreateEventPayload::new(
            String::from("Some Event Title"),
            String::from("Some event text"),
        )
        .add_tag(String::from("testing"));
        let result = client.post_event(&event).await;
        assert!(result.is_err());
        call.expect(1);
    }
}
