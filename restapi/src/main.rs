extern crate dotenv;
extern crate serde_derive;

mod errors;
mod routes;
mod views;
use actix_web::{App, HttpServer};
use dotenv::dotenv;
#[cfg(feature = "maria")]
use mariadb::db::connect;
#[cfg(feature = "postgres")]
use pg::db::connect;
use routes::{food, foods, nutrient_report, Context};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let pool = connect();
    let context = Context { db: pool.clone() };
    HttpServer::new(move || {
        App::new()
            .data(context.clone())
            .service(food)
            .service(foods)
            .service(nutrient_report)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
