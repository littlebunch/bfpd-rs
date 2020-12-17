# graphql-rs
A graphql server for the [USDA Branded Food Products](https://fdc.nal.usda.gov) dataset implemented with [Rust](https://www.rust-lang.org) using [Actix](https://actix.rs), [Juniper](https://docs.rs/juniper) and [Diesel](https://diesel.rs).  The data store can be [MariaDB](https://mariadb.com) or [PostgreSQL](https://www.postgresql.org).

A running instance of the server is available at [rs.littlebunch.com](https://rs.littlebunch.com/).  A docker image is available on [docker hub](https://hub.docker.com/repository/docker/littlebunch/graphql-rs).  

Feel free to take this project as a starting point for writing your own graphql service.

## What's here

[./src/graphql_schema.rs](https://github.com/littlebunch/graphql-rs/blob/master/src/graphql_schema.rs) -- graphql schema  
[./src/main.rs](https://github.com/littlebunch/graphql-rs/blob/master/src/main.rs) -- actix web server init and run  

## How to Build

This assumes you have a PostgreSQL or MariaDB database instance loaded and up and running.  Instructions for loading the database are provide [here].

### Step 1: Set-up your environment

If you haven't already, install the Rust [toolchain](https://www.rust-lang.org/tools/install) in your work environment 

### Step 2: Clone this repo

```bash
git clone git@github.com:littlebunch/bfpd-rs.git
```

### Step 3: Build the binary  

If you are using MariaDB:

```bash
cargo build --release --features mariadbfeature
```

If you are using PostgreSQL:

```bash
cargo build --release --features pgfeature
```

This will create the graphql-rs server in the top level ./target/release directory.

## How to run

### Step 1: Start the service

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

The client will be available at  http://localhost:8080/graphiql.

## Sample Queries

To get you started, here are some sample queries you can paste into the client of your choice, e.g. Insomnia, Postman or the local graphiql playground.  Use either http://localhost:8080/graphql or https://rs.littlebunch.com/graphql.

### Food UPC 000000018753 with all nutrient data

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

### Food UPC 000000018753 with nutrient data for Energy (Calories) (nutrient nbr = 208):

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

### Browse foods, sorted descending by food name:

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

### Search foods,  perform rudimentary searches using keywords in food descriptions and ingredients

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

###4 Browse foods by manufacturer 'General Mills, Inc'

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

### List nutrients sorted ascending by name

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
