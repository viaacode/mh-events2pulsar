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
    event_detail: Option<String>,
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

    #[test]
    fn test_trigger_export_request_v3_diff() {
        // Arrange
        let body = r##"<premis:event xmlns:premis="http://www.loc.gov/premis/v3">
            <premis:eventIdentifier>
                <premis:eventIdentifierType>MEDIAHAVEN_EVENT</premis:eventIdentifierType>
                <premis:eventIdentifierValue>111</premis:eventIdentifierValue>
            </premis:eventIdentifier>
            <premis:eventType>RECORDS.UPDATE</premis:eventType>
            <premis:eventDateTime>2024-08-12T15:01:08.751Z</premis:eventDateTime>
            <premis:eventDetailInformation>
                <premis:eventDetail />
                <premis:eventDetailExtension>
                    <mhs:Difference xmlns:mhs="https://zeticon.mediahaven.com/metadata/24.1/mhs/">
                        <mhs:MetadataFieldChange>
                            <mhs:DottedKey>Structural.FragmentStartFrames</mhs:DottedKey>
                            <mhs:ValueBefore>0000000000</mhs:ValueBefore>
                            <mhs:ValueAfter>0000000000</mhs:ValueAfter>
                        </mhs:MetadataFieldChange>
                        <mhs:MetadataFieldChange>
                            <mhs:DottedKey>RightsManagement.Permissions</mhs:DottedKey>
                            <mhs:ValueBefore>
                                <mh:Read xmlns:mh="https://zeticon.mediahaven.com/metadata/24.1/mh/">bce42e8b-7dfd-4b5d-96a1-2873aa310698</mh:Read>
                                <mh:Read xmlns:mh="https://zeticon.mediahaven.com/metadata/24.1/mh/">181aa247-7fbd-4a52-bb23-8a03c951bf16</mh:Read>
                                <mh:Read xmlns:mh="https://zeticon.mediahaven.com/metadata/24.1/mh/">ccf230e1-73bb-4efb-9472-e95bc00d6144</mh:Read>
                                <mh:Read xmlns:mh="https://zeticon.mediahaven.com/metadata/24.1/mh/">48447232-3997-45e6-b337-b4e51493a4e0</mh:Read>
                                <mh:Read xmlns:mh="https://zeticon.mediahaven.com/metadata/24.1/mh/">a5f55209-34ad-4d94-b96c-2061e03bf60a</mh:Read>
                                <mh:Write xmlns:mh="https://zeticon.mediahaven.com/metadata/24.1/mh/">bce42e8b-7dfd-4b5d-96a1-2873aa310698</mh:Write>
                                <mh:Write xmlns:mh="https://zeticon.mediahaven.com/metadata/24.1/mh/">181aa247-7fbd-4a52-bb23-8a03c951bf16</mh:Write>
                                <mh:Write xmlns:mh="https://zeticon.mediahaven.com/metadata/24.1/mh/">ccf230e1-73bb-4efb-9472-e95bc00d6144</mh:Write>
                                <mh:Write xmlns:mh="https://zeticon.mediahaven.com/metadata/24.1/mh/">48447232-3997-45e6-b337-b4e51493a4e0</mh:Write>
                                <mh:Write xmlns:mh="https://zeticon.mediahaven.com/metadata/24.1/mh/">a5f55209-34ad-4d94-b96c-2061e03bf60a</mh:Write>
                                <mh:Export xmlns:mh="https://zeticon.mediahaven.com/metadata/24.1/mh/">bce42e8b-7dfd-4b5d-96a1-2873aa310698</mh:Export>
                                <mh:Export xmlns:mh="https://zeticon.mediahaven.com/metadata/24.1/mh/">181aa247-7fbd-4a52-bb23-8a03c951bf16</mh:Export>
                                <mh:Export xmlns:mh="https://zeticon.mediahaven.com/metadata/24.1/mh/">ccf230e1-73bb-4efb-9472-e95bc00d6144</mh:Export>
                                <mh:Export xmlns:mh="https://zeticon.mediahaven.com/metadata/24.1/mh/">48447232-3997-45e6-b337-b4e51493a4e0</mh:Export>
                                <mh:Export xmlns:mh="https://zeticon.mediahaven.com/metadata/24.1/mh/">a5f55209-34ad-4d94-b96c-2061e03bf60a</mh:Export>
                            </mhs:ValueBefore>
                            <mhs:ValueAfter>
                                <mh:Read xmlns:mh="https://zeticon.mediahaven.com/metadata/24.1/mh/">ccf230e1-73bb-4efb-9472-e95bc00d6144</mh:Read>
                                <mh:Read xmlns:mh="https://zeticon.mediahaven.com/metadata/24.1/mh/">bce42e8b-7dfd-4b5d-96a1-2873aa310698</mh:Read>
                                <mh:Read xmlns:mh="https://zeticon.mediahaven.com/metadata/24.1/mh/">181aa247-7fbd-4a52-bb23-8a03c951bf16</mh:Read>
                                <mh:Write xmlns:mh="https://zeticon.mediahaven.com/metadata/24.1/mh/">ccf230e1-73bb-4efb-9472-e95bc00d6144</mh:Write>
                                <mh:Write xmlns:mh="https://zeticon.mediahaven.com/metadata/24.1/mh/">bce42e8b-7dfd-4b5d-96a1-2873aa310698</mh:Write>
                                <mh:Write xmlns:mh="https://zeticon.mediahaven.com/metadata/24.1/mh/">181aa247-7fbd-4a52-bb23-8a03c951bf16</mh:Write>
                                <mh:Export xmlns:mh="https://zeticon.mediahaven.com/metadata/24.1/mh/">ccf230e1-73bb-4efb-9472-e95bc00d6144</mh:Export>
                                <mh:Export xmlns:mh="https://zeticon.mediahaven.com/metadata/24.1/mh/">bce42e8b-7dfd-4b5d-96a1-2873aa310698</mh:Export>
                                <mh:Export xmlns:mh="https://zeticon.mediahaven.com/metadata/24.1/mh/">181aa247-7fbd-4a52-bb23-8a03c951bf16</mh:Export>
                            </mhs:ValueAfter>
                        </mhs:MetadataFieldChange>
                        <mhs:MetadataFieldChange>
                            <mhs:DottedKey>Structural.FragmentEndFrames</mhs:DottedKey>
                            <mhs:ValueBefore>0000000000</mhs:ValueBefore>
                            <mhs:ValueAfter>0000000000</mhs:ValueAfter>
                        </mhs:MetadataFieldChange>
                        <mhs:MetadataFieldChange>
                            <mhs:DottedKey>Dynamic.dc_types</mhs:DottedKey>
                            <mhs:ValueBefore />
                            <mhs:ValueAfter>
                                <mh:multiselect xmlns:mh="https://zeticon.mediahaven.com/metadata/24.1/mh/">Drama</mh:multiselect>
                            </mhs:ValueAfter>
                        </mhs:MetadataFieldChange>
                    </mhs:Difference>
                </premis:eventDetailExtension>
            </premis:eventDetailInformation>
            <premis:eventOutcomeInformation>
                <premis:eventOutcome>OK</premis:eventOutcome>
            </premis:eventOutcomeInformation>
            <premis:linkingAgentIdentifier>
                <premis:linkingAgentIdentifierType>MEDIAHAVEN_USER</premis:linkingAgentIdentifierType>
                <premis:linkingAgentIdentifierValue>7c741085-71db-4ab0-8d3d-f350a3fc4b1b</premis:linkingAgentIdentifierValue>
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
        assert_eq!(event.event_type, "RECORDS.UPDATE",);
        assert_eq!(
            event.event_timestamp,
            DateTime::<Utc>::from_str("2024-08-12T15:01:08.751Z").unwrap(),
        );
        assert_eq!(event.event_payload, body,);
        assert_eq!(event.to_xml(), body,);
    }
}
