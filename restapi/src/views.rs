use crate::routes::Context;
#[cfg(feature = "maria")]
use mariadb::models::*;
#[cfg(feature = "maria")]
use mariadb::Get;
#[cfg(feature = "postgres")]
use pg::models::*;
#[cfg(feature = "postgres")]
use pg::Get;

use serde::{Deserialize, Serialize};
use std::error::Error;
#[derive(Serialize, Deserialize, Debug)]
pub struct Foodview {
    pub publication_date: String,
    pub modified_date: String,
    pub available_date: String,
    pub upc: String,
    pub fdc_id: String,
    pub description: String,
    pub food_group: String,
    pub manufacturer: String,
    pub datasource: String,
    pub serving_size: Option<f64>,
    pub serving_unit: Option<String>,
    pub serving_description: Option<String>,
    pub country: Option<String>,
    pub ingredients: Option<String>,
    pub nutrient_data: Vec<Nutrientdataview>,
}

impl Foodview {
    pub fn build_view(
        fd: Vec<Food>,
        nids: &Vec<String>,
        context: &Context,
    ) -> Result<Vec<Foodview>, Box<dyn Error + Send + Sync>> {
        let conn = context.db.get().unwrap();
        let mut fv: Vec<Foodview> = Vec::new();
        for i in &fd {
            let f = &i;
            let mut fdv = Foodview::create(&f, &context);
            let nutform: Vec<NutrientdataForm> = f
                .get_nutrient_data(nids, &conn)
                .expect("error loading nutrient data");

            let mut ndv: Vec<Nutrientdataview> = Vec::new();
            for j in &nutform {
                let nf = &j;
                let mut nv = Nutrientdataview::create(&nf);
                nv.portion_value = match fdv.serving_size {
                    Some(x) => (x as f64 / 100.0) * nv.value,
                    None => 0.0,
                };
                ndv.push(nv);
            }
            fdv.nutrient_data = ndv;
            fv.push(fdv);
        }
        Ok(fv)
    }
    /// creates a new food view from a food
    pub fn create(f: &Food, context: &Context) -> Self {
        let conn = context.db.get().unwrap();
        Self {
            publication_date: f.publication_date.format("%Y-%m-%d").to_string(),
            modified_date: f.modified_date.format("%Y-%m-%d").to_string(),
            available_date: f.available_date.format("%Y-%m-%d").to_string(),
            upc: f.upc.to_string(),
            fdc_id: f.fdc_id.to_string(),
            description: f.description.to_string(),
            food_group: f.get_food_group_name(&conn).unwrap(),
            manufacturer: f.get_manufacturer_name(&conn).unwrap(),
            datasource: f.datasource.to_string(),
            serving_description: Some(
                f.serving_description
                    .as_ref()
                    .map(|n| n.to_string())
                    .unwrap_or("unknown".to_string()),
            ),
            serving_size: f.serving_size,
            serving_unit: Some(
                f.serving_unit
                    .as_ref()
                    .map(|n| n.to_string())
                    .unwrap_or("unknown".to_string()),
            ),
            country: Some(
                f.country
                    .as_ref()
                    .map(|n| n.to_string())
                    .unwrap_or("unknown".to_string()),
            ),
            ingredients: Some(
                f.ingredients
                    .as_ref()
                    .map(|n| n.to_string())
                    .unwrap_or("unknown".to_string()),
            ),
            nutrient_data: Vec::new(),
        }
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Nutrientdataview {
    pub value: f64,
    pub portion_value: f64,
    pub derivation: String,
    pub derivation_code: String,
    pub nutrient_no: String,
    pub nutrient: String,
    pub unit: String,
}

impl Nutrientdataview {
    pub fn create(n: &NutrientdataForm) -> Self {
        Self {
            value: n.value,
            portion_value: 0.0,
            nutrient_no: n.nutrient_no.to_string(),
            nutrient: n.nutrient.to_string(),
            unit: n.unit.to_string(),
            derivation: n.derivation.to_string(),
            derivation_code: n.derivation_code.to_string(),
        }
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Reportview {
    pub fdc_id: String,
    pub description: String,
    pub upc: String,
    pub serving_size: Option<f64>,
    pub serving_unit: Option<String>,
    pub serving_description: Option<String>,
    pub unit_value: f64,
    pub portion_value: f64,
}
impl Reportview {
    pub fn build_view(
        nd: Vec<Nutrientdata>,
        context: &Context,
    ) -> Result<Vec<Self>, Box<dyn Error + Send + Sync>> {
        let conn = context.db.get().unwrap();
        let mut rv: Vec<Self> = Vec::new();
        let mut f = Food::new();
        for i in &nd {
            f.id = i.food_id;
            let fv = f.get(&conn)?;
            rv.push(Reportview {
                fdc_id: fv[0].fdc_id.to_string(),
                upc: fv[0].upc.to_string(),
                description: fv[0].description.to_string(),
                serving_size: fv[0].serving_size,
                serving_description: Some(
                    fv[0]
                        .serving_description
                        .as_ref()
                        .map(|n| n.to_string())
                        .unwrap_or("unknown".to_string()),
                ),
                serving_unit: Some(
                    fv[0]
                        .serving_unit
                        .as_ref()
                        .map(|n| n.to_string())
                        .unwrap_or("unknown".to_string()),
                ),
                unit_value: i.value,
                portion_value: i.portion_value,
            })
        }
        Ok(rv)
    }
}
