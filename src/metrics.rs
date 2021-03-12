use crate::client::{Client, Error};
use serde::ser::SerializeSeq;
use serde::{Serialize, Serializer};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Type {
    Count,
    Gauge,
    Rate,
}

#[derive(Clone, Debug)]
pub struct Point {
    timestamp: u64,
    value: f64,
}

impl Point {
    pub fn new(timestamp: u64, value: f64) -> Self {
        Self { timestamp, value }
    }
}

impl Serialize for Point {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(2))?;
        seq.serialize_element(&self.timestamp)?;
        seq.serialize_element(&self.value)?;
        seq.end()
    }
}

/// # Examples
///
/// ```
/// use datadog_client::metrics::{Point, Serie, Type};
///
/// let serie = Serie::new("cpu.usage", Type::Gauge)
///     .set_host("raspberrypi")
///     .set_interval(42)
///     .set_points(vec![])
///     .add_point(Point::new(123456, 12.34))
///     .set_tags(vec![])
///     .add_tag(String::from("whatever:tag"));
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct Serie {
    // The name of the host that produced the metric.
    #[serde(skip_serializing_if = "Option::is_none")]
    host: Option<String>,
    // If the type of the metric is rate or count, define the corresponding interval.
    #[serde(skip_serializing_if = "Option::is_none")]
    interval: Option<i64>,
    // The name of the timeseries.
    metric: String,
    // Points relating to a metric. All points must be tuples with timestamp and a scalar value (cannot be a string).
    // Timestamps should be in POSIX time in seconds, and cannot be more than ten minutes in the future or more than one hour in the past.
    points: Vec<Point>,
    // A list of tags associated with the metric.
    tags: Vec<String>,
    // The type of the metric either count, gauge, or rate.
    #[serde(rename = "type")]
    dtype: Type,
}

impl Serie {
    pub fn new(metric: &str, dtype: Type) -> Self {
        Self {
            host: None,
            interval: None,
            metric: metric.to_string(),
            points: Vec::new(),
            tags: Vec::new(),
            dtype,
        }
    }
}

impl Serie {
    pub fn set_host(mut self, host: &str) -> Self {
        self.host = Some(host.to_string());
        self
    }

    pub fn set_interval(mut self, interval: i64) -> Self {
        self.interval = Some(interval);
        self
    }

    pub fn set_points(mut self, points: Vec<Point>) -> Self {
        self.points = points;
        self
    }

    pub fn add_point(mut self, point: Point) -> Self {
        self.points.push(point);
        self
    }
}

impl Serie {
    pub fn set_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn add_tag(mut self, tag: String) -> Self {
        self.tags.push(tag);
        self
    }
}

impl Client {
    /// Submit metrics
    ///
    /// https://docs.datadoghq.com/api/latest/metrics/#submit-metrics
    ///
    pub async fn post_metrics(&self, series: &[Serie]) -> Result<(), Error> {
        let payload = serde_json::json!({ "series": series });
        self.post("/api/v1/series", &payload).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::Config;
    use mockito::mock;

    #[test]
    fn serialize_point() {
        let point = Point::new(1234, 12.34);
        assert_eq!(serde_json::to_string(&point).unwrap(), "[1234,12.34]");
    }

    #[test]
    fn serialize_serie() {
        let serie = Serie::new("metric", Type::Count)
            .add_point(Point::new(1234, 1.234))
            .add_tag(String::from("tag"))
            .set_host("host");
        assert_eq!(
            serde_json::to_string(&serie).unwrap(),
            "{\"host\":\"host\",\"metric\":\"metric\",\"points\":[[1234,1.234]],\"tags\":[\"tag\"],\"type\":\"count\"}"
        );
    }

    #[tokio::test]
    async fn post_metrics_success() {
        let call = mock("POST", "/api/v1/series").with_status(202).create();
        let client = Client::new(Config::new(
            mockito::server_url(),
            String::from("fake-api-key"),
        ));
        let series = vec![Serie::new("something", Type::Gauge).add_point(Point::new(1234, 12.34))];
        let result = client.post_metrics(&series).await;
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
        let series = vec![Serie::new("something", Type::Gauge).add_point(Point::new(1234, 12.34))];
        let result = client.post_metrics(&series).await;
        assert!(result.is_err());
        call.expect(1);
    }
}
