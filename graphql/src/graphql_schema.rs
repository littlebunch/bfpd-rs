extern crate dotenv;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
use crate::views::*;
use juniper::{graphql_value, FieldError, FieldResult, IntoFieldError, RootNode};
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
use pg::{Browse, Count, Get};

const MAX_RECS: i32 = 150;
const DEFAULT_RECS: i32 = 50;

#[derive(Clone)]
pub struct Context {
    #[cfg(feature = "maria")]
    pub db: MysqlPool,
    #[cfg(feature = "postgres")]
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
                "Brand not found.",
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
        let mut fm = Brand::new();
        fm.owner = match filters.owners {
            None => "".to_string(),
            Some(m) => m,
        };
        if !fm.owner.is_empty() {
            let i = match fm.find_by_owner(&conn) {
                Ok(data) => data.id,
                Err(_e) => -1,
            };
            if i == -1 {
                return Err(CustomError::ManuNotFoundError.into_field_error());
            }
            food.brand_id = i;
        }
        let mut fgg = Foodgroup::new();
        fgg.description = match filters.food_group {
            None => "".to_string(),
            Some(m) => m,
        };
        if fgg.description.len() > 0 {
            let i = match fgg.find_by_description(&conn) {
                Ok(data) => data.id,
                Err(_e) => -1,
            };
            if i == -1 {
                return Err(CustomError::FoodGroupNotFoundError.into_field_error());
            }
            food.food_group_id = i;
        }
        food.ingredients = match filters.publication_date {
            None => None,
            Some(m) => Some(m),
        };
        food.description = match filters.query {
            None => "".to_string(),
            Some(m) => m,
        };
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

        let mut max = match browse.max {
            None => DEFAULT_RECS,
            Some(m) => m,
        };
        if max > MAX_RECS || max < 1 {
            return Err(CustomError::MaxValidationError.into_field_error());
        };
        let mut offset = match browse.offset {
            None => 0,
            Some(m) => m,
        };
        if offset < 0 {
            return Err(CustomError::OffsetError.into_field_error());
        }
        let mut order = match browse.order {
            None => "".to_string(),
            Some(m) => m,
        };
        let mut sort = match browse.sort {
            None => "".to_string(),
            Some(m) => m,
        };
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
        let filters = match browse.filters {
            None => Browsefilters::new(),
            Some(m) => m,
        };

        let mut fm = Brand::new();
        fm.owner = match filters.owners {
            None => "".to_string(),
            Some(m) => m,
        };
        if !fm.owner.is_empty() {
            let i = match fm.find_by_owner(&conn) {
                Ok(data) => data.id,
                Err(_e) => -1,
            };
            if i == -1 {
                return Err(CustomError::ManuNotFoundError.into_field_error());
            }
            food.brand_id = i;
        }
        // add food group filter if we have one
        let mut fgg = Foodgroup::new();
        fgg.description = match filters.food_group {
            None => "".to_string(),
            Some(m) => m,
        };
        if fgg.description.len() > 0 {
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
        food.ingredients = match filters.publication_date {
            None => None,
            Some(m) => Some(m),
        };
        // put any search terms into the food description field
        food.description = match filters.query {
            None => "".to_string(),
            Some(m) => m,
        };
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
    fn brands(
        context: &Context,
        mut max: i32,
        mut offset: i32,
        mut sort: String,
        order: String,
    ) -> FieldResult<Vec<BrandView>> {
        let conn = context.db.get().unwrap();
        if max > MAX_RECS || max < 1 {
            return Err(CustomError::MaxValidationError.into_field_error());
        }
        if offset < 0 {
            return Err(CustomError::OffsetError.into_field_error());
        }
        let m = Brand::new();
        let data = m.browse(max as i64, offset as i64, sort, order, &conn)?;
        let mut mv: Vec<BrandView> = Vec::new();
        for i in &data {
            mv.push(BrandView::create(&i));
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
    #[graphql(description=format!("Maximum records to return up to {}. Optional. Defaults to {}", MAX_RECS,DEFAULT_RECS))]
    pub max: Option<i32>,
    #[graphql(description = "Return records starting at an offset into the result set.  Optional.  Defaults to 0")]
    pub offset: Option<i32>,
    #[graphql(description = "Optional Sort by, one of: database id (default),description, upc or fdcId")]
    pub sort: Option<String>,
    #[graphql(description = "Optional Sort order, one of: asc (default) or desc")]
    pub order: Option<String>,
    #[graphql(description = "Optional filters to apply to the data")]
    pub filters: Option<Browsefilters>,
}
#[derive(juniper::GraphQLInputObject, Debug)]
pub struct Browsefilters {
    #[graphql(
        name = "pubdate",
        description = "Return records between two publication dates"
    )]
    pub publication_date: Option<String>,
    #[graphql(name = "fg", description = "Return records from specified food group")]
    pub food_group: Option<String>,
    #[graphql(
        name = "owner",
        description = "Return records from specified brand owner"
    )]
    pub owners: Option<String>,
    #[graphql(
        name = "query",
        description = "Filter on terms which appear in the food description and/or ingredients"
    )]
    pub query: Option<String>,
}
impl Browsefilters {
    fn new() -> Self {
        Self {
            publication_date: None,
            food_group: None,
            query: None,
            owners: None,
        }
    }
}
