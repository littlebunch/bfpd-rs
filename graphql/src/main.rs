extern crate dotenv;
extern crate serde_derive;
use actix_web::{get, post, web, App, Error, HttpResponse, HttpServer};
use dotenv::dotenv;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use std::env;
use std::sync::Arc;
mod graphql_schema;
mod views;
use crate::graphql_schema::{create_schema, Context, Schema};
#[cfg(feature = "maria")]
use mariadb::db::connect;
#[cfg(feature = "postgres")]
use pg::db::connect;
#[get("/graphiql")]
fn graphiql() -> HttpResponse {
    let url = match env::var("GRAPHIQL_URL") {
        Ok(x) => x,
        Err(_e) => "http://localhost:8000/graphql".to_string(),
    };
    //let html = graphiql_source("http://localhost:8080/graphql");
    let html = graphiql_source(&url);
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}
#[post("/graphql")]
async fn graphql(
    st: web::Data<Arc<Schema>>,
    ctx: web::Data<Context>,
    data: web::Json<GraphQLRequest>,
) -> Result<HttpResponse, Error> {
    let res = web::block(move || {
        let res = data.execute(&st, &ctx);
        Ok::<_, serde_json::error::Error>(serde_json::to_string(&res)?)
    })
    .await
    .map_err(Error::from)?;
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(res))
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let pool = connect();
    let schema_context = Context { db: pool.clone() };
    let schema = std::sync::Arc::new(create_schema());
    HttpServer::new(move || {
        App::new()
            .data(schema.clone())
            .data(schema_context.clone())
            .service(graphql)
            .service(graphiql)
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await
}
