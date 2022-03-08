use pulsar::{
    producer::{self, SendFuture}, Error as PulsarError, MultiTopicProducer, Pulsar,
    SerializeMessage, TokioExecutor,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub data: String,
}

impl SerializeMessage for Message {
    fn serialize_message(input: Self) -> Result<producer::Message, PulsarError> {
        Ok(producer::Message {
            payload: input.data.into_bytes(),
            ..Default::default()
        })
    }
}

pub struct PulsarClient {
    pub producer: MultiTopicProducer<TokioExecutor>,
}

impl PulsarClient {
    pub async fn new() -> Result<Self, PulsarError> {
        // Deserialize XML to struct
        let pulsar: Pulsar<_> = Pulsar::builder("pulsar://127.0.0.0:6650", TokioExecutor).build().await?;
        let producer = pulsar
            .producer()
            .with_name("mh-events2pulsar")
            .build_multi_topic();
        Ok(PulsarClient { producer: producer })
    }

    pub async fn send_message(
        &mut self,
        topic: String,
        data: String,
    ) -> Result<SendFuture, pulsar::Error> {
        self.producer
            .send(
                topic,
                Message {
                    data: data,
                },
            )
            .await
    }
}
