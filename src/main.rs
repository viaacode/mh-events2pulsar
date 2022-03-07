use std::io::BufWriter;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use xmltree::Element;

use mh_events2pulsar::*;

async fn livez() -> impl Responder {
    HttpResponse::Ok()
}

async fn event(req_body: String) -> impl Responder {
    let xml_result = Element::parse(req_body.as_bytes());
    match xml_result {
        Ok(xml_tree) => {
            // One ore more premis events are contained in an Events node.
            for child in xml_tree.children {
                if child.as_element().unwrap().name == "event" {
                    // Write child element (= the premis event) to a String.
                    let buf = Vec::new();
                    let mut writer = BufWriter::new(buf);
                    child.as_element().unwrap().write(&mut writer).unwrap();
                    let premis_event_xml = String::from_utf8(writer.into_inner().unwrap()).unwrap();
                    // Create the Event struct
                    let premis_event = Event::new(&premis_event_xml);

                    let xml = premis_event.to_xml();
                    println!("{xml}");
                }
            }
        }
        Err(_e) => {
            //handle_error(e);
            //consumer.reject(delivery, false)?;
        }
    }
    HttpResponse::Ok()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/livez", web::get().to(livez))
            .route("/event", web::post().to(event))
    })
    .bind(("127.0.0.1", 8080))?
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

        let mut app = test::init_service(App::new().route("/event", web::post().to(event))).await;
        // Act
        let req = test::TestRequest::post()
            .uri("/event")
            .set_payload(body)
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        // Assert
        assert!(resp.status().is_success());
        let body = to_bytes(resp.into_body()).await.unwrap();
        assert!(from_utf8(&body).unwrap().is_empty());
    }
}
