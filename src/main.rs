use actix_web::{get, post, App, HttpResponse, HttpServer, Responder};


#[get("/livez")]
async fn livez() -> impl Responder {
    HttpResponse::Ok()
}

#[post("/event")]
async fn event(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(livez)
            .service(event)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}