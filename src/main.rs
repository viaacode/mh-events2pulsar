use std::io::BufWriter;
use std::sync::{Arc, Mutex};

use actix_web::{
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use log::{debug, error, info};
use xmltree::Element;

mod pulsar_client;
use crate::pulsar_client::PulsarClient;
use mh_events2pulsar::{Config, Event};

async fn livez() -> impl Responder {
    HttpResponse::Ok()
}

/// The event endpoint.
///
/// Parse incoming premis events and send them to a pulsar topic defined by the event type.
///
/// # Arguments
///
/// * `req_body` - The request body of the post call.
/// * `pulsar_client` - The shared Pulsar client state used to send messages to a topic.
async fn events(req_body: String, pulsar_client: web::Data<Mutex<PulsarClient>>) -> impl Responder {
    debug!("Incoming event: {:?}", req_body);
    let xml_result = Element::parse(req_body.as_bytes());
    match xml_result {
        Ok(xml_tree) => {
            // One ore more premis events are contained in an Events node.
            for child in xml_tree.children {
                if child.as_element().unwrap().name == "event" {
                    // Write child element (= the premis event) to a String.
                    let buf = Vec::new();
                    let mut writer = BufWriter::new(buf);
                    match child.as_element().unwrap().write(&mut writer) {
                        Ok(_) => {}
                        Err(e) => {
                            error!("Error: {}", e);
                            return HttpResponse::InternalServerError().body(e.to_string());
                        }
                    }
                    match String::from_utf8(writer.into_inner().unwrap()) {
                        Ok(premis_event_xml) => {
                            // Create the Event struct
                            let premis_event = Event::new(&premis_event_xml);
                            // The topic part in: persistent://{tenant}/{namespace}/{topic}.
                            let topic = format!("be.mediahaven.{}", &premis_event.event_type.to_lowercase());
                            // Send message to Pulsar topic.
                            let send_message_result = pulsar_client
                                .lock()
                                .unwrap()
                                .send_message(&topic, &premis_event)
                                .await;
                            match send_message_result {
                                Ok(_) => {
                                    info!("Sent event on topic: '{}'.", &premis_event.event_type);
                                }
                                Err(e) => {
                                    error!("Error: {}", e);
                                    return HttpResponse::InternalServerError().body(e.to_string());
                                }
                            }
                        }
                        Err(e) => {
                            error!("Error: {}", e);
                            return HttpResponse::InternalServerError().body(e.to_string());
                        }
                    }
                }
            }
        }
        Err(e) => {
            error!("Error: {}", e);
            return HttpResponse::BadRequest().body(e.to_string());
        }
    }
    HttpResponse::Ok().finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //Initialize the logger
    env_logger::init();

    // Get our configuration from the environment
    // The necessary environment variables can be found in the `.env` file
    let config = match envy::from_env::<Config>() {
        Ok(config) => config,
        Err(error) => panic!("{:#?}", error),
    };

    // Intanstiate a Pulsar client to pass as a shared mutable state.
    let pulsar_client = PulsarClient::new(&config).await.unwrap();
    let client = Arc::new(Mutex::new(pulsar_client));
    info!("Started the Pulsar client.");
    // Create the HTTP server.
    info!("Starting the HTTP server on '127.0.0.1:8080'.");
    HttpServer::new(move || {
        App::new()
            .app_data(Data::from(client.clone()))
            .route("/livez", web::get().to(livez))
            .route("/events", web::post().to(events))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::body::to_bytes;
    use actix_web::{test, web, App};
    use std::str::from_utf8;

    #[actix_web::test]
    async fn test_livez() {
        // Arrange
        let mut app = test::init_service(App::new().route("/livez", web::get().to(livez))).await;
        // Act
        let req = test::TestRequest::with_uri("/livez").to_request();
        let resp = test::call_service(&mut app, req).await;
        // Assert
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_event() {
        // Arrange
        // XML body to receive
        let body = r##"<events>
            <premis:event xmlns:premis="info:lc/xmlns/premis-v2">
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
            </premis:event>
        </events>"##;
        // Mock the Pulsar client
        // TODO

        // Create a HTTP test service
        let mut app = test::init_service(App::new().route("/events", web::post().to(events))).await;
        // Act
        let req = test::TestRequest::post()
            .uri("/events")
            .set_payload(body)
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        // Assert
        assert!(resp.status().is_success());
        let body = to_bytes(resp.into_body()).await.unwrap();
        assert!(from_utf8(&body).unwrap().is_empty());
    }
}
