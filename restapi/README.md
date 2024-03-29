# restapi
A REST server for the [USDA Branded Food Products](https://fdc.nal.usda.gov) dataset implemented with [Rust](https://www.rust-lang.org) using [Actix](https://actix.rs) and [Diesel](https://diesel.rs).  The data store can be [MariaDB](https://mariadb.com) or [PostgreSQL](https://www.postgresql.org).  

Feel free to take this project as a starting point for writing your own service.

## What's here

[./src/errors.rs](https://github.com/littlebunch/bfpd-rs/blob/master/restapi/src/errors.rs) -- wrapper for HTTP error responses  
[./src/routes.rs](https://github.com/littlebunch/bfpd-rs/blob/master/restapi/src/routes.rs)  -- the request handlers  
[./src/views.rs](https://github.com/littlebunch/bfpd-rs/blob/master/restapi/src/views.rs)  -- data returned by a query, sort of like business objects  
[./src/main.rs](https://github.com/littlebunch/bfpd-rs/blob/master/restapi/src/main.rs) -- actix web server init and run    

## How to Build

This assumes you have a PostgreSQL or MariaDB database instance loaded and up and running.  Instructions for loading the database are provide [here].

### Step 1: Set-up your environment

If you haven't already, install the Rust [toolchain](https://www.rust-lang.org/tools/install) in your work environment 

### Step 2: Clone this repo

```bash
git clone git@github.com:littlebunch/bfpd-rs.git
```

### Step 3: Build the binary  

From the restapi directory, if you are using MariaDB:

```bash
cargo build --release --features maria
```

If you are using PostgreSQL:

```bash
cargo build --release --features postgres
```

This will create the restapi server binary in the top level ./target/release directory.

## How to run

### Step 1: Start the service

You need to set an environment variable named DATABASE_URL.  It generally makes sense to create an .env file in the root path of your project which gets loaded at start-up:

```bash
DATABASE_URL=postgres://user:password@localhost/bfpd
```

Then run the server from the project root (the path where cargo.toml is located):

```bash
./target/release/restapi
```

The client will be available at  http://localhost:8080/.

## Sample Queries

### Find food UPC 000000018753 with all nutrient data

```bash
curl -H "Content-type:application/json" http://localhost:8080/food/000000018753  

where:  
id is upc, fdc_id or database id  
```

### List foods with all nutrient data  

```bash
curl -H "Content-type:application/json" -D '{max:50,offset:0,sort:"description",order:"asc"}' http://localhost:8080/foods 

where:  
sort = order by description, fdc_id or upc  
order = asc or desc  
max =  number of foods to return  
offset = start browse on record offset  
```

### List foods ordered by nutrient value

```bash
curl -H "Content-type:application/json" -D '{sort:"portion_value",max:50,offset:0,nutrient:"208",vmin:10,vmax:100}' http://localhost:8080/report 

where:  
sort = portion value or 100 g equivalent value  
max =  number of foods to return  
nutrient = nutrient number to report  
vmin = minimum nutrient value to return
vmax = maximum nutrient value to return  
```
