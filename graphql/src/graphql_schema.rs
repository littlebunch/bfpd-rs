extern crate dotenv;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
#[cfg(feature="maria")]
use mariadb::db::MysqlPool;
#[cfg(feature="maria")]
use mariadb::models::*;
#[cfg(feature="maria")]
use mariadb::{Browse, Count, Get};
#[cfg(feature="postgres")]
use pg::db::PgPool;
#[cfg(feature="postgres")]
use pg::models::*;
#[cfg(feature="postgres")]
use pg::{Browse, Count, Get};
use juniper::{graphql_value, FieldError, FieldResult, IntoFieldError, RootNode};
use crate::views::*;

const MAX_RECS: i32 = 150;

#[derive(Clone)]
pub struct Context {
    #[cfg(feature="maria")]
    pub db: MysqlPool,
    #[cfg(feature="postgres")]
    pub db: PgPool,
}

impl juniper::Context for Context {}

enum CustomError {
    MaxValidationError,
    OffsetError,
    FoodSortError,
    FoodGroupNotFoundError,
    ManuNotFoundError,
}

impl juniper::IntoFieldError for CustomError {
    fn into_field_error(self) -> FieldError {
        match self {
            CustomError::MaxValidationError => FieldError::new(
                format!(
                    "max parameter exceeds allowed amounts.  Enter 1 to {}",
                    MAX_RECS
                ),
                graphql_value!({
                    "type": "MAX_ERROR"
                }),
            ),
            CustomError::OffsetError => FieldError::new(
                "offset parameter must be greater than 1",
                graphql_value!({
                    "type": "OFFSET_ERROR"
                }),
            ),
            CustomError::FoodSortError => FieldError::new(
                "sort parameter not recognized.  try 'description','fdcid', 'upc' or 'id'",
                graphql_value!({
                    "type": "SORT_ERROR"
                }),
            ),
            CustomError::FoodGroupNotFoundError => FieldError::new(
                "Food group not found.",
                graphql_value!({
                    "type": "NOT_FOUND_ERROR"
                }),
            ),
            CustomError::ManuNotFoundError => FieldError::new(
                "Manufacturer not found.",
                graphql_value!({
                    "type": "NOT_FOUND_ERROR"
                }),
            ),
        }
    }
}
pub struct QueryRoot;
#[juniper::object(Context = Context)]
impl QueryRoot {
    // count foods in a query
    fn foods_count(context: &Context, mut filters: Browsefilters) -> FieldResult<Querycount> {
        use std::convert::TryFrom;
        let mut food = Food::new();
        let conn = context.db.get().unwrap();
        if !filters.manufacturers.is_empty() {
            let mut fm = Manufacturer::new();
            fm.name = filters.manufacturers;
            let i = match fm.find_by_name(&conn) {
                Ok(data) => data.id,
                Err(_e) => -1,
            };
            if i == -1 {
                return Err(CustomError::ManuNotFoundError.into_field_error());
            }
            food.manufacturer_id = i;
        }
        if !filters.food_group.is_empty() {
            let mut fgg = Foodgroup::new();
            fgg.description = filters.food_group;
            if fgg.description.len() == 0 {
                fgg.description = String::from("Unknown");
            }
            let i = match fgg.find_by_description(&conn) {
                Ok(data) => data.id,
                Err(_e) => -1,
            };
            if i == -1 {
                return Err(CustomError::FoodGroupNotFoundError.into_field_error());
            }
            food.food_group_id = i;
        }
        if !filters.publication_date.is_empty() {
            food.ingredients = Some(filters.publication_date)
        }
        food.description = filters.query;
        let c64 = food.query_count(&conn)?;
        let c32 = i32::try_from(c64)?;
        Ok(Querycount { count: c32 })
    }
    async fn foods(
        context: &Context,
        mut browse: Browsequery,
        nids: Vec<String>,
    ) -> FieldResult<Vec<Foodview>> {
        let conn = context.db.get().unwrap();

        let mut max = browse.max;
        if max > MAX_RECS || max < 1 {
            return Err(CustomError::MaxValidationError.into_field_error());
        }
        let mut offset = browse.offset;
        if offset < 0 {
            return Err(CustomError::OffsetError.into_field_error());
        }
        let mut order = browse.order;
        let mut sort = browse.sort;
        if sort.is_empty() {
            sort = "id".to_string();
        }
        sort = sort.to_lowercase();
        sort = match &*sort {
            "description" => "description".to_string(),
            "id" => "id".to_string(),
            "fdcid" => "fdcId".to_string(),
            "upc" => "upc".to_string(),
            _ => "".to_string(),
        };
        if sort.is_empty() {
            return Err(CustomError::FoodSortError.into_field_error());
        }
        let mut food = Food::new();
        // stash filters into the Food struct, this is ugly but helps keep things simple
        // for users and the model
        if !browse.filters.manufacturers.is_empty() {
            let mut fm = Manufacturer::new();
            fm.name = browse.filters.manufacturers;
            let i = match fm.find_by_name(&conn) {
                Ok(data) => data.id,
                Err(_e) => -1,
            };
            if i == -1 {
                return Err(CustomError::ManuNotFoundError.into_field_error());
            }
            food.manufacturer_id = i;
        }
        // add food group filter if we have one
        if !browse.filters.food_group.is_empty() {
            let mut fgg = Foodgroup::new();
            fgg.description = browse.filters.food_group;
            if fgg.description.len() == 0 {
                fgg.description = String::from("Unknown");
            }
            let i = match fgg.find_by_description(&conn) {
                Ok(data) => data.id,
                Err(_e) => -1,
            };
            if i < 1 {
                return Err(CustomError::FoodGroupNotFoundError.into_field_error());
            }
            food.food_group_id = i;
        }
        // stash publication date filter into food ingredients
        // ugly but expedient
        if !browse.filters.publication_date.is_empty() {
            food.ingredients = Some(browse.filters.publication_date)
        }
        // put any search terms into the food description field
        food.description = browse.filters.query;
       
        let data = food.browse(max as i64, offset as i64, sort, order, &conn)?;
        Ok(Foodview::build_view(data, &nids, context))
    }
    async fn food(context: &Context, fid: String, nids: Vec<String>) -> FieldResult<Vec<Foodview>> {
        let conn = context.db.get().unwrap();
        let mut food = Food::new();

        if fid.len() >= 10 {
            food.upc = fid;
        } else {
            food.fdc_id = fid;
        }

        let data = food.get(&conn)?;
        Ok(Foodview::build_view(data, &nids, context))
    }
    fn nutrient(context: &Context, nno: String) -> FieldResult<Vec<Nutrientview>> {
        let conn = context.db.get().unwrap();
        let mut n = Nutrient::new();
        n.nutrientno = nno;
        let nut = n.get(&conn)?;
        let mut nv: Vec<Nutrientview> = Vec::new();
        for i in &nut {
            let nv1 = &i;
            nv.push(Nutrientview::create(nv1));
        }
        Ok(nv)
    }
    fn nutrients(
        context: &Context,
        mut max: i32,
        mut offset: i32,
        mut sort: String,
        mut order: String,
        nids: Vec<String>,
    ) -> FieldResult<Vec<Nutrientview>> {
        let conn = context.db.get()?;
        let mut b = false;
        if max > MAX_RECS || max < 1 {
            return Err(CustomError::MaxValidationError.into_field_error());
        }
        if offset < 0 {
            return Err(CustomError::OffsetError.into_field_error());
        }
        let n = Nutrient::new();

        let data = n.browse(max as i64, offset as i64, sort, order, &conn)?;
        let mut nv: Vec<Nutrientview> = Vec::new();
        for i in &data {
            let nv1 = &i;
            nv.push(Nutrientview::create(nv1));
        }

        Ok(nv)
    }
    fn manufacturers(
        context: &Context,
        mut max: i32,
        mut offset: i32,
        mut sort: String,
        order: String,
    ) -> FieldResult<Vec<ManufacturerView>> {
        let conn = context.db.get().unwrap();
        if max > MAX_RECS || max < 1 {
            return Err(CustomError::MaxValidationError.into_field_error());
        }
        if offset < 0 {
            return Err(CustomError::OffsetError.into_field_error());
        }
        let m = Manufacturer::new();
        let data = m.browse(max as i64, offset as i64, sort, order, &conn)?;
        let mut mv: Vec<ManufacturerView> = Vec::new();
        for i in &data {
            mv.push(ManufacturerView::create(&i));
        }
        Ok(mv)
    }
    fn food_groups(
        context: &Context,
        mut max: i32,
        mut offset: i32,
        mut sort: String,
        order: String,
    ) -> FieldResult<Vec<FoodgroupView>> {
        let conn = context.db.get().unwrap();
        if max > MAX_RECS || max < 1 {
            return Err(CustomError::MaxValidationError.into_field_error());
        }
        if offset < 0 {
            return Err(CustomError::OffsetError.into_field_error());
        }
        let fg = Foodgroup::new();
        let data = fg.browse(max as i64, offset as i64, sort, order, &conn)?;
        let mut fgv: Vec<FoodgroupView> = Vec::new();
        for i in &data {
            let fgv1 = &i;
            fgv.push(FoodgroupView::create(fgv1));
        }
        Ok(fgv)
    }
}
pub struct MutationRoot;

#[juniper::object(Context = Context)]
impl MutationRoot {
    fn create_food_not_implemented(context: &Context) -> String {
        String::from("not implemented")
    }
}
pub type Schema = RootNode<'static, QueryRoot, MutationRoot>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, MutationRoot {})
}

#[derive(juniper::GraphQLInputObject, Debug)]
#[graphql(
    name = "BrowseRequest",
    description = "Input object for defining a foods browse query"
)]
pub struct Browsequery {
    #[graphql(description=format!("Maximum records to return up to {}", MAX_RECS))]
    pub max: i32,
    #[graphql(description = "Return records starting at an offset into the result set")]
    pub offset: i32,
    #[graphql(description = "Sort by, one of: database id (default),description, upc or fdcId")]
    pub sort: String,
    #[graphql(description = "Sort order, one of: asc (default) or desc")]
    pub order: String,
    #[graphql(description = "Filters to apply to the data")]
    pub filters: Browsefilters,
}
#[derive(juniper::GraphQLInputObject, Debug)]
pub struct Browsefilters {
    #[graphql(
        name = "pubdate",
        description = "Return records between two publication dates"
    )]
    pub publication_date: String,
    #[graphql(name = "fg", description = "Return records from specified food group")]
    pub food_group: String,
    #[graphql(
        name = "manu",
        description = "Return records from specified manufacturer"
    )]
    pub manufacturers: String,
    #[graphql(
        name = "query",
        description = "Filter on terms which appear in the food description and/or ingredients"
    )]
    pub query: String,
}
