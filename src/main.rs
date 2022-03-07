use actix_web::{web, App, HttpResponse, HttpServer, Responder};

async fn livez() -> impl Responder {
    HttpResponse::Ok()
}

async fn event(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
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
    use std::str::from_utf8;
    use actix_web::body::to_bytes;
    use actix_web::{test, web, App};

    #[actix_web::test]
    async fn test_livez() {
        let mut app = test::init_service(App::new().route("/livez", web::get().to(livez))).await;
        let req = test::TestRequest::with_uri("/livez").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_event() {
        let mut app = test::init_service(App::new().route("/event", web::post().to(event))).await;
        let req = test::TestRequest::post().uri("/event").set_payload("body").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());
        let body = to_bytes(resp.into_body()).await.unwrap();
        assert_eq!(
            "body",
            from_utf8(&body).unwrap()
        );
    }
}

