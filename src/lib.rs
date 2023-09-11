use chrono::{DateTime, Utc};
use std::str;

use quick_xml::de::from_str;
use serde::{Deserialize, Serialize};

// Config
#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_pulsar_host")]
    pub pulsar_host: String,
    #[serde(default = "default_pulsar_port")]
    pub pulsar_port: String,
    #[serde(default = "default_pulsar_namespace")]
    pub pulsar_namespace: String,
}

fn default_pulsar_host() -> String {
    String::from("localhost")
}

fn default_pulsar_port() -> String {
    String::from("6650")
}

fn default_pulsar_namespace() -> String {
    String::from("default")
}

// XML structs
#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    #[serde(rename = "eventIdentifier")]
    event_identifier: EventIdentifier,
    #[serde(rename = "eventType")]
    pub event_type: String,
    #[serde(rename = "eventDateTime")]
    pub event_timestamp: DateTime<Utc>,
    #[serde(rename = "eventDetail")]
    event_detail: String,
    #[serde(rename = "eventOutcomeInformation")]
    event_outcome_information: EventOutcomeInformation,
    #[serde(rename = "linkingAgentIdentifier")]
    linking_agent_identifier: LinkingAgentIdentifier,
    #[serde(rename = "linkingObjectIdentifier")]
    linking_object_identifier: Vec<LinkingObjectIdentifier>,
    #[serde(skip_deserializing)]
    pub event_payload: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventIdentifier {
    #[serde(rename = "eventIdentifierType")]
    event_identifier_type: String,
    #[serde(rename = "eventIdentifierValue")]
    event_identifier_value: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventOutcomeInformation {
    #[serde(rename = "eventOutcome")]
    event_outcome: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LinkingAgentIdentifier {
    #[serde(rename = "linkingAgentIdentifierType")]
    linking_agent_identifier_type: String,
    #[serde(rename = "linkingAgentIdentifierValue")]
    linking_agent_identifier_value: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LinkingObjectIdentifier {
    #[serde(rename = "linkingObjectIdentifierType")]
    linking_object_identifier_type: String,
    #[serde(rename = "linkingObjectIdentifierValue")]
    linking_object_identifier_value: String,
}

impl Event {
    pub fn new(body: &str) -> Event {
        // Deserialize XML to struct
        let mut event: Event = from_str(body).unwrap();
        // Add the body XMl as payload to the struct
        event.event_payload = body.to_string();
        event
    }

    pub fn to_xml(&self) -> String {
        self.event_payload.clone()
    }

    pub fn subject(&self) -> String {
        for x in &self.linking_object_identifier {
            if x.linking_object_identifier_type == "EXTERNAL_ID" {
                return x.linking_object_identifier_value.clone();
            }
        }
        // No subject found. Should not happen, but let's not panic.
        String::from("no_subject_found")
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    #[test]
    fn test_trigger_export_request() {
        // Arrange
        let body = r##"<premis:event xmlns:premis="info:lc/xmlns/premis-v2">
            <premis:eventIdentifier>
                <premis:eventIdentifierType>MEDIAHAVEN_EVENT</premis:eventIdentifierType>
                <premis:eventIdentifierValue>111</premis:eventIdentifierValue>
            </premis:eventIdentifier>
            <premis:eventType>FLOW.ARCHIVED</premis:eventType>
            <premis:eventDateTime>2019-03-30T05:28:40Z</premis:eventDateTime>
            <premis:eventDetail>Ionic Defibulizer</premis:eventDetail>
            <premis:eventOutcomeInformation>
                <premis:eventOutcome>OK</premis:eventOutcome>
            </premis:eventOutcomeInformation>
            <premis:linkingAgentIdentifier>
                <premis:linkingAgentIdentifierType>MEDIAHAVEN_USER</premis:linkingAgentIdentifierType>
                <premis:linkingAgentIdentifierValue>703a53d2-dc66-4eb2-ab7f-73d5fd228852</premis:linkingAgentIdentifierValue>
            </premis:linkingAgentIdentifier>
            <premis:linkingObjectIdentifier>
                <premis:linkingObjectIdentifierType>MEDIAHAVEN_ID</premis:linkingObjectIdentifierType>
                <premis:linkingObjectIdentifierValue>a1b2c3</premis:linkingObjectIdentifierValue>
            </premis:linkingObjectIdentifier>
            <premis:linkingObjectIdentifier>
                <premis:linkingObjectIdentifierType>EXTERNAL_ID</premis:linkingObjectIdentifierType>
                <premis:linkingObjectIdentifierValue>a1</premis:linkingObjectIdentifierValue>
            </premis:linkingObjectIdentifier>
        </premis:event>"##;
        // Act
        let event = Event::new(body);
        // Assert
        assert_eq!(event.event_type, "FLOW.ARCHIVED",);
        assert_eq!(
            event.event_timestamp,
            DateTime::<Utc>::from_str("2019-03-30T05:28:40Z").unwrap(),
        );
        assert_eq!(event.event_payload, body,);
        assert_eq!(event.to_xml(), body,);
    }
}
