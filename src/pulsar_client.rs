use chrono::{DateTime, Utc};
use pulsar::{
    producer::{self, SendFuture},
    Error as PulsarError, MultiTopicProducer, Pulsar, SerializeMessage, TokioExecutor,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, to_vec};
use std::collections::HashMap;
use uuid::Uuid;

use mh_events2pulsar::{Config, Event};

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub data: String,
    pub event_time: DateTime<Utc>,
    pub subject: String,
}

impl SerializeMessage for Message {
    fn serialize_message(input: Self) -> Result<producer::Message, PulsarError> {
        let event_time = input.event_time;
        let properties = HashMap::from([
            (String::from("type"), String::from("structured")),
            (String::from("source"), String::from("mh-events2pulsar")),
            (String::from("subject"), input.subject),
            (String::from("outcome"), String::from("success")),
            (
                String::from("correlation_id"),
                Uuid::new_v4().to_simple().to_string(),
            ),
            (String::from("specversion"), String::from("1.0")),
            (
                String::from("content_type"),
                String::from("application/cloudevents+json; charset=utf-8"),
            ),
        ]);
        let payload = json!({
            "datacontenttype": "application/json",
            "data": {
                "premis": input.data,
            },
            "type": &properties["type"],
            "source": &properties["source"],
            "subject": &properties["subject"],
            "outcome": &properties["outcome"],
            "correlation_id": &properties["correlation_id"],
            "specversion": &properties["specversion"],
            "content_type": &properties["content_type"],
        });

        Ok(producer::Message {
            payload: to_vec(&payload).unwrap(),
            event_time: Some(event_time.timestamp_millis() as u64),
            properties,
            ..Default::default()
        })
    }
}

pub struct PulsarClient {
    pub producer: MultiTopicProducer<TokioExecutor>,
    pub namespace: String,
}

impl PulsarClient {
    pub async fn new(config: &Config) -> Result<Self, PulsarError> {
        // Deserialize XML to struct
        let addr = format!("pulsar://{}:{}", config.pulsar_host, config.pulsar_port);
        let pulsar: Pulsar<_> = Pulsar::builder(addr, TokioExecutor).build().await?;
        let namespace = config.pulsar_namespace.clone();
        let producer = pulsar
            .producer()
            .with_name("mh-events2pulsar")
            .build_multi_topic();
        Ok(PulsarClient { producer, namespace })
    }

    pub async fn send_message(
        &mut self,
        topic: &str,
        event: &Event,
    ) -> Result<SendFuture, pulsar::Error> {
        self.producer
            .send(
            format!("persistent://public/{}/{}", self.namespace, topic),
                Message {
                    data: event.to_xml(),
                    event_time: event.event_timestamp,
                    subject: event.subject(),
                },
            )
            .await
    }
}
