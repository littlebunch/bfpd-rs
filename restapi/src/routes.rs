use crate::errors::{CustomError, ErrorResponse};
use crate::views::Foodview;
use actix_web::{get, web, web::Data, Error, HttpResponse};
#[cfg(feature = "maria")]
use mariadb::db::MysqlPool;
#[cfg(feature = "maria")]
use mariadb::models::*;
#[cfg(feature = "maria")]
use mariadb::{Browse, Count, Get};
#[cfg(feature = "postgres")]
use pg::db::PgPool;
#[cfg(feature = "postgres")]
use pg::models::*;
#[cfg(feature = "postgres")]
use pg::{Browse, Get};
use serde::{Deserialize, Serialize};
pub const MAX_RECS: i32 = 150;

#[derive(Clone)]
pub struct Context {
    #[cfg(feature = "maria")]
    pub db: MysqlPool,
    #[cfg(feature = "postgres")]
    pub db: PgPool,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Browsequery {
    max: Option<i32>,
    offset: Option<i32>,
    sort: Option<String>,
    order: Option<String>,
}

#[get("/food/{fid}")]
pub async fn food(ctx: Data<Context>, id: web::Path<String>) -> Result<HttpResponse, Error> {
    let conn = ctx.db.get().expect("couldn't get DB connection from pool");
    let mut f = Food::new();
    let fid = id.to_string();
    if fid.len() >= 10 {
        f.upc = fid;
    } else {
        f.fdc_id = fid;
    }
    let data = web::block(move || f.get(&conn)).await.unwrap();
    let nids: Vec<String> = Vec::new();
    Ok(web::block(move || Foodview::build_view(data, &nids, &ctx))
        .await
        .map(|fvs| HttpResponse::Ok().json(fvs))
        .map_err(|_| HttpResponse::InternalServerError())?)
}
#[get("/foods")]
pub async fn foods(
    ctx: Data<Context>,
    browse: web::Query<Browsequery>,
) -> Result<HttpResponse, Error> {
    let conn = ctx.db.get().expect("couldn't get DB connection from pool");
    let mut errs: Vec<ErrorResponse> = Vec::new();
    let max = match browse.max {
        None => 50,
        _ => browse.max.unwrap(),
    };
    let mut sort = match browse.sort {
        None => "id".to_string(),
        _ => browse.sort.as_ref().unwrap().to_string(),
    };
    sort = sort.to_lowercase();
    sort = match &*sort {
        "description" => "description".to_string(),
        "id" => "id".to_string(),
        "fdcid" => "fdcId".to_string(),
        "upc" => "upc".to_string(),
        _ => "".to_string(),
    };
    if sort.is_empty() {
        errs.push(ErrorResponse::new(CustomError::MaxValidationError));
    }
    let order = match browse.order {
        None => "asc".to_string(),
        _ => browse.order.as_ref().unwrap().to_string(),
    };
    if order.to_uppercase() != "ASC" && order.to_uppercase() != "DESC" {
        errs.push(ErrorResponse::new(CustomError::OrderError));
    }
    let offset = match browse.offset {
        None => 0,
        _ => browse.offset.unwrap(),
    };
    if offset < 0 {
        errs.push(ErrorResponse::new(CustomError::OffsetError));
    }
    if max > MAX_RECS || max < 1 {
        errs.push(ErrorResponse::new(CustomError::MaxValidationError));
    }
    if errs.len() > 0 {
        return HttpResponse::BadRequest().json(errs).await;
    }
    let mut f = Food::new();
    f.description = "".to_string();
    let data = web::block(move || f.browse(max as i64, offset as i64, sort, order, &conn))
        .await
        .unwrap();
    let nids: Vec<String> = Vec::new();
    Ok(web::block(move || Foodview::build_view(data, &nids, &ctx))
        .await
        .map(|fvs| HttpResponse::Ok().json(fvs))
        .map_err(|_| HttpResponse::InternalServerError())?)
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Reportquery {
    max: Option<i32>,
    offset: Option<i32>,
    sort: Option<String>,
    order: Option<String>,
    nutrient: String,
    vmin: f64,
    vmax: f64,
}

#[get("/report")]
pub async fn nutrient_report(
    ctx: Data<Context>,
    rq: web::Query<Reportquery>,
) -> Result<HttpResponse, Error> {
    let conn = ctx.db.get().expect("couldn't get DB connection from pool");
    let mut errs: Vec<ErrorResponse> = Vec::new();
    let mut n = Nutrient::new();
    let max = match rq.max {
        None => 50,
        _ => rq.max.unwrap(),
    };
    if max > MAX_RECS || max < 1 {
        errs.push(ErrorResponse::new(CustomError::MaxValidationError));
    }
    let offset = match rq.offset {
        None => 0,
        _ => rq.offset.unwrap(),
    };
    if offset < 0 {
        errs.push(ErrorResponse::new(CustomError::OffsetError));
    }
    n.nutrientno = rq.nutrient.to_string();
    let n = match n.find_by_no(&conn) {
        Ok(data) => data.id,
        Err(_e) => -1,
    };
    if n == -1 {
        errs.push(ErrorResponse::new(CustomError::MaxValidationError));
    }
    let mut sort = match rq.sort {
        None => "value".to_string(),
        _ => rq.sort.as_ref().unwrap().to_string(),
    };
    sort = sort.to_lowercase();
    sort = match &*sort {
        "portion" => "portion".to_string(),
        "value" => "value".to_string(),
        _ => "".to_string(),
    };
    if sort.is_empty() {
        errs.push(ErrorResponse::new(CustomError::ReportSortError));
    }
    let mut order = match rq.order {
        None =>"asc".to_string(),
        _ => rq.order.as_ref().unwrap().to_string(),
    };
    order = order.to_lowercase();
    order = match &*order {
        "asc" => "asc".to_string(),
        _ => "desc".to_string(),
    };
    if rq.vmin > rq.vmax {
        errs.push(ErrorResponse::new(CustomError::MinMaxError));
    }
    if errs.len() > 0 {
        return HttpResponse::BadRequest().json(errs).await;
    }
    let f = Food::new();
    let data = web::block(move || {
        f.get_report(max as i64, offset as i64, sort, order, rq.vmin, rq.vmax, n, &conn)
    })
    .await
    .unwrap();
    Ok(HttpResponse::Ok().json(data))
}
