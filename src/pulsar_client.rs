use chrono::{DateTime, Utc};
use pulsar::{
    producer::{self, SendFuture},
    Error as PulsarError, MultiTopicProducer, Pulsar, SerializeMessage, TokioExecutor,
};
use serde::{Deserialize, Serialize};
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
        let payload = input.data.into_bytes();
        let event_time = input.event_time;
        let properties = HashMap::from([
            ("type".to_string(), "structured".to_string()),
            ("source".to_string(), "mh-events2pulsar".to_string()),
            ("subject".to_string(), input.subject),
            ("outcome".to_string(), "OK".to_string()),
            (
                "correlation_id".to_string(),
                Uuid::new_v4().to_simple().to_string(),
            ),
            ("specversion".to_string(), "1.0".to_string()),
            (
                "datacontenttype".to_string(),
                "application/cloudevents+json; charset=utf-8".to_string(),
            ),
        ]);
        Ok(producer::Message {
            payload: payload,
            event_time: Some(event_time.timestamp_millis() as u64),
            properties: properties,
            ..Default::default()
        })
    }
}

pub struct PulsarClient {
    pub producer: MultiTopicProducer<TokioExecutor>,
}

impl PulsarClient {
    pub async fn new(config: &Config) -> Result<Self, PulsarError> {
        // Deserialize XML to struct
        let addr = format!("pulsar://{}:{}", config.pulsar_host, config.pulsar_port);
        let pulsar: Pulsar<_> = Pulsar::builder(addr, TokioExecutor).build().await?;
        let producer = pulsar
            .producer()
            .with_name("mh-events2pulsar")
            .build_multi_topic();
        Ok(PulsarClient { producer: producer })
    }

    pub async fn send_message(
        &mut self,
        topic: &str,
        event: &Event,
    ) -> Result<SendFuture, pulsar::Error> {
        self.producer
            .send(
                topic,
                Message {
                    data: event.to_xml(),
                    event_time: event.event_timestamp,
                    subject: event.subject(),
                },
            )
            .await
    }
}
