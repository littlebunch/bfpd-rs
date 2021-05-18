use crate::diesel::{QueryDsl,RunQueryDsl,ExpressionMethods};
use crate::models::{Derivation, Food, Foodgroup, Brand, Nutrient, Nutrientdata};
use crate::Get;
use chrono::NaiveDateTime;
use csv::{Reader, StringRecord};
use diesel::dsl::insert_into;
use diesel::pg::PgConnection;
/// thanks to @andrewleverette https://github.com/andrewleverette/rust_csv_examples
use std::error::Error;
use std::fmt;
const BATCH_SIZE: usize = 2000;
/// A simple error handler structure
#[derive(Debug)]
struct IndexError(String);

impl fmt::Display for IndexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Index Error: {}", self.0)
    }
}

impl Error for IndexError {}

/// Internal data set to make aggregation simpler
#[derive(Debug)]
struct DataSet {
    /// Header row of CSV file
    headers: StringRecord,

    /// Records from CSV file
    records: Vec<StringRecord>,
}

impl DataSet {
    /// Creates a new data set
    fn new(headers: StringRecord, records: Vec<StringRecord>) -> Self {
        DataSet { headers, records }
    }

    /// Finds the index of a column given the column name
    ///
    /// # Arguments
    ///
    /// * `key` -> The column name
    ///
    /// # Errors
    ///
    /// An error occurs if column name does not exist.
    fn key_index(&self, key: &str) -> Result<usize, Box<dyn Error>> {
        match self.headers.iter().position(|column| column == key) {
            Some(index) => Ok(index),
            None => Err(Box::new(IndexError(format!(
                "Column '{}' does not exist.",
                key
            )))),
        }
    }

    /// Sort data records by the given index.
    ///
    /// # Errors
    ///
    /// An error occurs if the index is out of bounds
    fn sort_by_index(&mut self, index: usize) -> Result<(), Box<dyn Error>> {
        if index >= self.headers.len() {
            Err(Box::new(IndexError(format!(
                "Index '{}' out of bounds",
                index
            ))))
        } else {
            self.records.sort_by(|a, b| a[index].cmp(&b[index]));
            Ok(())
        }
    }
}

/// This trait defines aggregation methods for the internal data set
trait Aggregate {
    fn inner_join(&mut self, right: &mut Self, key: &str) -> Result<DataSet, Box<dyn Error>>;
}

impl Aggregate for DataSet {
    /// Performs an inner join on two data sets, where `self` is the left table.
    ///
    /// # Arguments
    ///
    /// * `right` -> The right data set for the join
    /// * `key` -> The column name to join on
    fn inner_join(&mut self, right: &mut Self, key: &str) -> Result<DataSet, Box<dyn Error>> {
        // Get column index
        let left_index = self.key_index(key)?;
        let right_index = right.key_index(key)?;

        // Merge headers
        let headers = StringRecord::from(
            self.headers
                .iter()
                .chain(right.headers.iter())
                .collect::<Vec<&str>>(),
        );

        let mut records = vec![];

        if self.records.is_empty() || right.records.is_empty() {
            return Ok(DataSet::new(headers, records));
        }

        // Sort data sets by the column index
        // Required to for this sort algorithm
        self.sort_by_index(left_index)?;
        right.sort_by_index(right_index)?;

        let mut left_cursor = 0;
        let mut right_cursor = 0;

        while left_cursor < self.records.len() && right_cursor < right.records.len() {
            // If two fields match, merge fields into a single record
            // and add to records vector
            // If they don't match and the left value is less then right value advance the left cursor
            // else advance the right cursor
            if self.records[left_cursor][left_index] == right.records[right_cursor][right_index] {
                let record = StringRecord::from(
                    self.records[left_cursor]
                        .iter()
                        .chain(right.records[right_cursor].iter())
                        .collect::<Vec<&str>>(),
                );

                records.push(record);

                // Since data sets are sorted
                // Advance cursor through right data set to
                // see if there are matches
                let mut k = right_cursor + 1;
                while k < right.records.len()
                    && self.records[left_cursor][left_index] == right.records[k][right_index]
                {
                    let record = StringRecord::from(
                        self.records[left_cursor]
                            .iter()
                            .chain(right.records[k].iter())
                            .collect::<Vec<&str>>(),
                    );

                    records.push(record);
                    k += 1;
                }
                left_cursor += 1;
            } else if self.records[left_cursor][left_index]
                < right.records[right_cursor][right_index]
            {
                left_cursor += 1;
            } else {
                right_cursor += 1;
            }
        }

        Ok(DataSet::new(headers, records))
    }
}

/// Reads csv data from a file and returns a DataSet
fn read_from_file(path: &str) -> Result<DataSet, Box<dyn Error>> {
    let mut reader = Reader::from_path(path)?;

    let headers = reader.headers()?.clone();

    let records = reader
        .records()
        .collect::<Result<Vec<StringRecord>, csv::Error>>()?;

    Ok(DataSet { headers, records })
}

/// Foodcsv for deserializing the merged food.csv and branded.csv output
#[derive(Deserialize, Debug)]
struct Foodcsv {
    fdc_id: String,              //r[0]
    datatype: String,            //r[1]
    description: String,         //r[2]
    blank: String,               //r[3]
    date_published: String,      //r[4]
    fdc_id_two: String,          //r[5]
    owner: String,        //r[6]
    brand: String,          //r[7]
    subbrand: String,       //r[8]
    upc: String,                 //r[7]
    ingredients: String,         //r[8]
    not_significant: String,    //r[9]
    serving_size: String,        //r[10]
    serving_unit: String,        //r[11]
    serving_description: String, //r[12]
    foodgroup: String,           //r[13]
    datasource: String,          //r[14]
    date_modified: String,       //r[15]
    date_available: String,      //r[16]
    country: String,
}

impl Foodcsv {
    /// Creates a Food struct from the contents of Foodcsv
    fn create_food(&self, conn: &PgConnection) -> Result<Food, Box<dyn Error>> {
        let mut f = Food::new();
        let adate = match self.date_available.is_empty() {
            true => String::from("1970-01-01 19:00:00"),
            false => self.date_available.to_string() + " 19:00:00",
        };
        let pdate = match self.date_published.is_empty() {
            true => String::from("1970-01-01 19:00:00"),
            false => self.date_published.to_string() + " 19:00:00",
        };
        let mdate = match self.date_modified.is_empty() {
            true => String::from("1970-01-01 19:00:00"),
            false => self.date_modified.to_string() + " 19:00:00",
        };
        let s: f64 = match self.serving_size.parse() {
            Ok(data) => data,
            Err(_e) => 0.0,
        };
        f.upc = self.upc.to_string();
        f.fdc_id = self.fdc_id.to_string();
        f.description = self.description.to_string();
        f.datasource = self.datasource.to_string();
        f.serving_unit = Some(self.serving_unit.to_string());
        f.serving_description = Some(self.serving_description.to_string());
        f.serving_size = Some(s); //convert
        f.country = Some(self.country.to_string());
        f.ingredients = Some(self.ingredients.to_string());
        f.publication_date = NaiveDateTime::parse_from_str(&pdate, "%Y-%m-%d %H:%M:%S")?;
        f.modified_date = NaiveDateTime::parse_from_str(&mdate, "%Y-%m-%d %H:%M:%S")?;
        f.available_date = NaiveDateTime::parse_from_str(&adate, "%Y-%m-%d %H:%M:%S")?;
        f.food_group_id = self.create_foodgroup_id(conn)?;
        f.brand_id = self.create_brand_id(conn)?;

        Ok(f)
    }
    
    /// Returns the database id for a brand owner as identified by the owner name.  
    /// Inserts a new owner row if id is not found
    fn create_brand_id(&self, conn: &PgConnection) -> Result<i32, Box<dyn Error>> {
        use crate::schema::brands::dsl::*;
        let mut b = Brand::new();
        b.owner = self.owner.to_string();
        if b.owner == "" {
            b.owner = String::from("Unknown");
        }
       let mut i = match b.find_by_owner(conn) {
            Ok(data) => data.id,
            Err(_e) => -1,
        };
        if i == -1 {
            insert_into(brands)
                .values(owner.eq(b.owner))
                .execute(conn)?;
            i = self.create_brand_id(conn)?;
        }
        
        Ok(i)
    }
    /// Returns the database id for a food group as identified by the food group description
    /// Inserts a new row if the description is not in the table
    fn create_foodgroup_id(&self, conn: &PgConnection) -> Result<i32, Box<dyn Error>> {
        use crate::schema::food_groups::dsl::*;
        let mut fg = Foodgroup::new();
        fg.description = self.foodgroup.to_string();
        if fg.description.len() == 0 {
            fg.description = String::from("Unknown");
        }
        let mut i = match fg.find_by_description(conn) {
            Ok(data) => data.id,
            Err(_e) => -1,
        };
        if i == -1 {
            insert_into(food_groups)
                .values(description.eq(fg.description))
                .execute(conn)?;
            i = self.create_foodgroup_id(conn)?;
        }
        Ok(i)
    }
}
/// A BFPD food is created from 2 csv files:  food.csv and branded.csv.
/// Using code adapted from https://github.com/andrewleverette/rust_csv_examples, these
/// two files are sorted and aggregated then deserialized into a data transfer struct and finally
/// into an insertable Food struct for the database.
pub fn process_foods(path: String, conn: &PgConnection) -> Result<usize, Box<dyn Error>> {
    use crate::schema::foods::dsl::*;
    // Read food.csv
    let foodfile = format!("{}{}", path, "food.csv");
    let mut foodcsv = read_from_file(&foodfile)?;

    let brandedfile = format!("{}{}", path, "branded_food.csv");
    // Read branded_food.csv
    let mut branded = read_from_file(&brandedfile)?;

    // Aggregate the 2 files using inner_join
    let result = foodcsv.inner_join(&mut branded, "fdc_id")?;
    let mut fv = Vec::new();
    let mut fcsv: Foodcsv;
    let mut count: usize = 0;
    // deserialize the foodcsv collection into a Food vec and
    // insert into the database BATCH_SIZE records at a time.
    for r in &result.records {
        fcsv = r.deserialize(None).expect("Can't deserialize");
        let f = fcsv.create_food(conn).expect("Can't create food from csv");
        match () {
            #[cfg(not(feature = "pg"))]
            () => fv.push((
                upc.eq(f.upc),
                fdc_id.eq(f.fdc_id),
                description.eq(f.description),
                datasource.eq(f.datasource),
                serving_unit.eq(f.serving_unit),
                serving_description.eq(f.serving_description),
                serving_size.eq(f.serving_size),
                country.eq(f.country),
                ingredients.eq(f.ingredients),
                publication_date.eq(f.publication_date),
                available_date.eq(f.available_date),
                modified_date.eq(f.modified_date),
                food_group_id.eq(f.food_group_id),
                brand_id.eq(f.brand_id),
            )),
            #[cfg(feature = "pg")]
            () => fv.push(f),
        };
        if fv.len() % BATCH_SIZE == 0 {
            count += insert_into(foods).values(&fv).execute(conn).unwrap();
            fv.clear();
        }
    }
    count += insert_into(foods).values(&fv).execute(conn).unwrap();
    fv.clear();
    Ok(count)
}
/// NutdataCsv for deserializing the csv
#[derive(Deserialize, Debug)]
struct NutdataCsv {
    id: i32,
    fdc_id: String,
    nutrient_id: i32,
    amount: f64,
    data_points: String,
    derivation_id: i32,
    min: String,
    max: String,
    median: String,
    footnote: String,
    min_year: String,
}
impl NutdataCsv {
    // transfers a NutdataCsv to a Nutrientdata struct
    fn create_nutdata(&self, fid: i32) -> Nutrientdata {
        Nutrientdata {
            id: 0,
            value: self.amount,
            portion_value: self.amount,
            standard_error: None,
            minimum: None,
            maximum: None,
            median: None,
            derivation_id: self.derivation_id,
            nutrient_id: self.nutrient_id,
            food_id: fid,
        }
    }
}
/// Deserializes the food_nutrient.csv data into NutdataCsv structs then into Nutrientdata structs
/// which are then inserted into the nutrient_data table
pub fn process_nutdata(path: String, conn: &PgConnection) -> Result<usize, Box<dyn Error>> {
    use crate::schema::nutrient_data::dsl::*;
    let mut count: usize = 0;
    let ndfile = format!("{}{}", path, "food_nutrient.csv");
    let ndcsv = read_from_file(&ndfile)?;
    let mut ndsv: NutdataCsv;
    let mut nds = Vec::new();
    let mut fid: i32 = 0;
    let mut ofdc_id: String = String::from("z");
    let mut f = Food::new();
    for n in ndcsv.records {
        ndsv = n.deserialize(None).expect("Can't deserialize csv");
        // batch csv by food fdc_id for efficient database look-up
        if ndsv.fdc_id != ofdc_id {
            f.fdc_id = ndsv.fdc_id.to_string();
            let fv = f.get(conn).expect("Cannot get food id");
            fid = fv[0].id;
            f.serving_size=fv[0].serving_size;
            ofdc_id = ndsv.fdc_id.to_string();
        }
        
        let mut nd = ndsv.create_nutdata(fid);
        nd.portion_value = match f.serving_size {
            Some(x) => (x as f64 / 100.0) * nd.value,
            None => 0.0,
        };
         // jump through some hoops to get nutrient_id
        // necessary because some nutrient_id's in the csv are
        // nutrientno's and others are nutrient ids
        let mut nut=Nutrient::new();
        nut.nutrientno = ndsv.nutrient_id.to_string();
        // if we have a nutrientno then get the nutrient id
        // we can end up with a value of -1 if the nutrient id
        // can't be found
        let nid = match nut.find_by_no(conn)  {
            Ok(data)=> data.id,
            Err(_e)=> {
                use crate::schema::nutrients::dsl::*;
                match nutrients.filter(id.eq(&ndsv.nutrient_id)).first::<Nutrient>(conn) {
                    Ok(data)=> data.id,
                    Err(_e)=> -1,
                }
            }

        };
        if nid == -1 {
            println!("Cannot find nutrient value for {} fdc_id = {}",ndsv.nutrient_id,ndsv.fdc_id);
            continue
        }
        nd.nutrient_id = nid;
        match () {
            #[cfg(not(feature = "postgres"))]
            () => nds.push((
                value.eq(nd.value),
                portion_value.eq(nd.portion_value),
                minimum.eq(nd.minimum),
                maximum.eq(nd.maximum),
                median.eq(nd.median),
                derivation_id.eq(nd.derivation_id),
                nutrient_id.eq(nd.nutrient_id),
                food_id.eq(nd.food_id),
            )),
            #[cfg(feature = "postgres")]
            () => nds.push(nd),
        };
        // insert the Nutrientdata when vec contains BATCH_SIZE recs
        if nds.len() % BATCH_SIZE == 0 {
            count += insert_into(nutrient_data)
                .values(&nds)
                .execute(conn)
                .unwrap();
            nds.clear();
        }
    }
    // empty the vec
    count += insert_into(nutrient_data)
        .values(&nds)
        .execute(conn)
        .unwrap();
    Ok(count)
}
#[derive(Deserialize, Debug)]
struct Nutcsv {
    id: i32,
    name: String,
    unit: String,
    nbr: String,
    order: String,
}
impl Nutcsv {
    fn create_nutrient(&self) -> Result<Nutrient, Box<dyn Error>> {
        let n = Nutrient {
            id: self.id,
            description: self.name.to_string(),
            unit: self.unit.to_string(),
            nutrientno: self.nbr.to_string(),
        };
        Ok(n)
    }
}
/// Inserts nutrients csv into the database
pub fn process_nutrients(path: String, conn: &PgConnection) -> Result<usize, Box<dyn Error>> {
    use crate::schema::nutrients::dsl::*;
    let nutfile = format!("{}{}", path, "nutrient.csv");
    let recs = read_from_file(&nutfile)?;
    let mut ncsv: Nutcsv;
    let mut nuts: Vec<Nutrient> = Vec::new();
    for n in recs.records {
        ncsv = n.deserialize(None).expect("Can't deserialize csv");
        let nut = ncsv
            .create_nutrient()
            .expect("Can't create nutrient record");
        nuts.push(nut);
    }
    Ok(insert_into(nutrients).values(&nuts).execute(conn).unwrap())
}
#[derive(Deserialize, Debug)]
struct Dervcsv {
    id: i32,
    code: String,
    description: String,
    source: String,
}
impl Dervcsv {
    fn create_derivation(&self) -> Result<Derivation, Box<dyn Error>> {
        let d = Derivation {
            id: self.id,
            code: self.code.to_string(),
            description: self.description.to_string(),
        };
        Ok(d)
    }
}
/// Inserts derivation csv into the database
pub fn process_derivations(path: String, conn: &PgConnection) -> Result<usize, Box<dyn Error>> {
    use crate::schema::derivations::dsl::*;
    let dervfile = format!("{}{}", path, "food_nutrient_derivation.csv");
    let recs = read_from_file(&dervfile)?;
    let mut dcsv: Dervcsv;
    let mut dervs: Vec<Derivation> = Vec::new();
    for d in recs.records {
        dcsv = d.deserialize(None).expect("Can't deserialize csv");
        let derv = dcsv
            .create_derivation()
            .expect("Can't create derivation record");
        dervs.push(derv);
    }
    Ok(insert_into(derivations)
        .values(&dervs)
        .execute(conn)
        .unwrap())
}
