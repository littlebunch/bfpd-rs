# bfpd-rs

[Rust](https://www.rust-lang.org) libraries and binaries to load [USDA Branded Food Products](https://fdc.nal.usda.gov) CSV into a PostgreSQL or Mariadb/Mysql database for serving as a backend for a [GraphQL](https://graphql.org) server.

## What's here

The repository is organized as a [Cargo workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.htmlhttps://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html).  

[./mariadb](https://github.com/littlebunch/graphql-rs/blob/master/src/csv.rs) -- Mariadb/Mysql library  
[./pg](https://github.com/littlebunch/graphql-rs/blob/master/src/db.rs) -- Postgresql library  
[./graphql](https://github.com/littlebunch/graphql-rs/blob/master/src/graphql_schema.rs) -- graphql server  
[./ingest-csv](https://github.com/littlebunch/graphql-rs/blob/master/src/bin/ingest-csv.rs) -- cli utility for importing the USDA csv files into the database  
[./data/pg](https://github.com/littlebunch/graphql-rs/tree/master/datab/pg) -- Diesel migration scripts to create the PostgreSQL database and schema.rs  
[./data/mariadb](https://github.com/littlebunch/graphql-rs/tree/master/database/mariadb) -- Diesel migration scripts to create the Mariadb database and schema.rs  

## How to Set-up the Database  

### Step 1. Create an empty schema  

For PostgreSQL:

```bash
createdb bfpd
```

For MariaDB:  

```bash
mysql -u user -p -e"create schema bfpd;"
```

### Step 2: Load the data  

At this point, you have a couple of options:  1) download a dump of a recent version of the Branded Food Products database from [https://go.littlebunch.com/posgresql](https://go.littlebunch.com/postgres.sql.gz) for a PostgreSQL dump or [https://go.littlebunch.com/mariadb](https://go.littlebunch.com/bfpd-2020-11-07.sql.gz) for a Mariadb dump and restore to your local instance or 2) build the database from the ground-up by importing the USDA csv files using the provided ingest-csv command line utility or 2) .  

### Using a dump file  

If you are using the first option, download the dump files and restore to your database:  

For postgreSQL:

```bash
psql -U [user] bfpd < [downloaded.sql]
```

For MariaDB:  

```bash
mysql -u [user] < [downloaded.sql]
```

###  Using the ingest-csv utility  

### Step 1: Set-up your environment

If you haven't already, install the Rust [toolchain](https://www.rust-lang.org/tools/install) in your work environment as well as a recent version of [PostgreSQL](https://www.postgresql.org/download/) or [Mariadb](https://mariadb.com))

### Step 2: Clone this repo  

```bash
git clone git@github.com:littlebunch/graphql-rs.git
```


  The utility is a first draft and assumes you are importing into an empty database.

1. Download and unzip the latest csv from the [FDC website](https://fdc.nal.usda.gov/download-datasets.html) into a directory of your choice.  You will need the Branded Foods and Supporting data for All Downloads zip files:

```bash
wget https://fdc.nal.usda.gov/fdc-datasets/FoodData_Central_branded_food_csv_2020-04-29.zip
```

```bash
wget https://fdc.nal.usda.gov/fdc-datasets/FoodData_Central_Supporting_Data_csv_2020-04-29.zip
```

Use the Diesel migration script to create an empty database.

For PostgreSQL:  

```bash
psql -U user -W bfpd < database/pg/up.sql
```

For Mariadb:  

```bash
mysql -u user -p bfpd < database/mariadb/up.sql
```

Note: You can use the up.sql and down.sql scripts to create a [diesel migration](https://diesel.rs/guides/getting-started/).  This is probably more trouble than it's worth unless you need to change the schema or just want to learn a bit more about diesel migrations.

4. Load the data by pointing the program to the full path containing the csv:

```bash
./target/release/ingest-cvs -p /path/to/csv/
```

The load takes about 3-10 minutes depending on your hardware.  Note:  you need to set a DATABASE_URL variable as described in Step 2 below before running the ingest-csv program.

### Step 4: Build the binaries

If you are using PostgreSQL:  

```bash
cargo build --release --features pgfeature
```

If you are using MariaDB or MySQL:  

```bash
cargo build --release --features mariadbfeature
```

This will create the graphql-rs server in the ./target/release directory.  If you are importing USDA csv, then build the cli utility for doing that:

```bash
cargo build --release --bin ingest-csv
```

### Step 5: Start the service

You need to set a couple of environment variables.  It generally makes sense to put them in an .env file in the root path of your project which gets loaded at start-up:

```bash
DATABASE_URL=postgres://user:password@localhost/bfpd
GRAPHIQL_URL=http://localhost:8080/graphql
```

Then run the server from the project root (the path where cargo.toml is located):

```bash
./target/release/graphql-rs
```

or start a Docker instance:

```bash
docker run --rm -it -p 8080:8080 --env-file=/full/path/to/.env littlebunch/graphql-rs
```

The client will be available at  <http://localhost:8080/graphiql>.

## Sample Queries
To get you started, here are some sample queries you can paste into the client of your choice, e.g. Insomnia, Postman or the local graphiql playground.  Use either <http://localhost:8080/graphql> or <https://rs.littlebunch.com/graphql>.

#### Food UPC 000000018753 with all nutrient data

```bash
{
  food(fid:"000000018753", nids: []) {
    upc
    description
    servingSize
    servingDescription
    servingUnit
    nutrientData {
      value
      portionValue
      nutrientNo
      nutrient
      unit
    }
  }
}
```

#### Food UPC 000000018753 with nutrient data for Energy (Calories) (nutrient nbr = 208)

```bash
{
  food(fid:"000000018753", nids: ["208"]) {
    upc
    description
    servingSize
    servingDescription
    servingUnit
    nutrientData {
      value
      portionValue
      nutrientNo
      nutrient
      unit
    }
  }
}
```

#### Browse foods, sorted descending by food name

```bash
{
  foods(browse:{max: 150, offset: 0, sort: "description", order:"desc",filters:{query:"",manu:"",fg:"",pubdate:""}}, nids: []) {
    upc
    description
    manufacturer
    food
    ingredients
    foodGroup
    nutrientData {
      portionValue
      nutrientNo
      nutrient
      unit
    }
  }
}
```

#### Search foods,  perform rudimentary searches using keywords in food descriptions and ingredients

```bash
{
  foods(browse: {max: 150, offset: 0, sort: "", order: "", filters: {query:"BTY CRK HLO KTY COOKIE",pubdate: "", fg: "", manu: ""}}, nids: ["208"]) {
    upc
    description
    publicationDate
    manufacturer
    foodGroup
    ingredients
    nutrientData {
      portionValue
      nutrientNo
      nutrient
      unit
    }
  }
}
```

### Count foods returned from a search

```bash
{
  foodsCount( filters: {query:"BTY CRK HLO KTY COOKIE",pubdate: "", fg: "", manu: ""}) {
   count
  }
}
```

### Browse foods by manufacturer 'General Mills, Inc'

```bash
{
  foods(browse: {max: 150, offset: 0, sort: "", order: "", filters: {query:"",pubdate: "", fg:"", manu: "General Mills, Inc."}}, nids: ["208"]) {
    upc
    description
    publicationDate
    manufacturer
    foodGroup
    ingredients
  }
}
```

#### List nutrients sorted ascending by name:

```bash
{
  nutrients(max: 100, offset: 0, sort: "name", order: "asc", nids: []) {
    nbr
    name
    unit
  }
}
```

### List food groups sorted ascending by group

```bash
{
  foodGroups(max:125,offset:0,sort:"group",order:"asc") {
    id
    group
  }
}
```

### List food manufacturers (owners) sorted ascending by name

```bash
{
  foodGroups(max:150,offset:0,sort:"name",order:"asc") {
    id
    name
  }
}
```
