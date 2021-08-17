# bfpd-rs

[Rust](https://www.rust-lang.org) PostgreSQL and MariaDB libraries for processing [USDA Branded Food Products](https://fdc.nal.usda.gov) data.  The Cargo workspace also includes binaries for a CLI utility for loading CSV and REST API and [GraphQL](https://graphql.org) servers.

## What's here

The repository is organized as a [Cargo workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.htmlhttps://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html).  

[./mariadb](https://github.com/littlebunch/bfpd-rs/tree/master/mariadb) -- MariadDB library  
[./pg](https://github.com/littlebunch/bfpd-rs/tree/master/pg) -- PostgreSQL library  
[./graphql](https://github.com/littlebunch/bfpd-rs/tree/master/graphql) -- graphql server  
[./restapi](https://github.com/littlebunch/bfpd-rs/tree/master/restapi) -- REST server  
[./ingest-csv](https://github.com/littlebunch/bfpd-rs/tree/master/ingest-csv) -- cli utility for importing the USDA csv files into the database  
[./data/pg](https://github.com/littlebunch/bfpd-rs/tree/master/data/pg) -- Diesel migration scripts to create the PostgreSQL database and schema.rs  
[./data/mariadb](https://github.com/littlebunch/bfpd-rs/tree/master/data/mariadb) -- Diesel migration scripts to create the MariaDB database and schema.rs  

## How to Set-up the Database  

You can choose either PostgreSQL (13 recommended) or MariaDB 10.  In either case, it's assumed you have a running instance of one or the other.

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

At this point, you have a couple of options:  1) download a dump of a recent version of the Branded Food Products database from [https://go.littlebunch.com/posgresql](https://go.littlebunch.com/bfpd5-postgresql-20210618.dmp.gz) for a PostgreSQL dump or [https://go.littlebunch.com/mariadb](https://go.littlebunch.com/bfpd5-mysql-20210617.sql.gz) for a MariaDB dump and restore to your local instance or 2) build the database from the ground-up by importing the USDA csv files using the provided ingest-csv command line utility.  

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

### Using the ingest-csv utility  

01. Build the binary
Instructions for building the ingest-csv executable are provided in the [ingest-csv/README.md](https://github.com/littlebunch/bfpd-rs/tree/master/ingest-csv).

02. Download and unzip the latest csv from the [FDC website](https://fdc.nal.usda.gov/download-datasets.html) into a directory of your choice.  You will need the Branded Foods and Supporting data for All Downloads zip files:

    ```bash
    wget https://fdc.nal.usda.gov/fdc-datasets/FoodData_Central_branded_food_csv_2021-04-28.zip
    ```

    ```bash
    wget https://fdc.nal.usda.gov/fdc-datasets/FoodData_Central_Supporting_Data_csv_2021-04-28.zip
    ```

03. Use the Diesel migration scripts in the data directory to create an empty database

      For PostgreSQL:  

      ```bash
      psql -U user -W bfpd < data/pg/up.sql
      ```

      For MariaDB:  

      ```bash
      mysql -u user -p bfpd < data/mariadb/up.sql
      ```

      Note: You can use the up.sql and down.sql scripts to create a [diesel migration](https://diesel.rs/guides/getting-started/).  This is probably more trouble than it's worth unless you need to change the schema or just want to learn a bit more about diesel migrations.

04. Load the data by pointing the program to the full path containing the csv

    ```bash
    ./target/release/ingest-cvs -p /path/to/csv/
    ```

The load takes about 30 minutes depending on your hardware.  Note:  you need to set a DATABASE_URL variable as described in Step 4 in the ingest-csv README. 

### Step 3 Publish the data 

You can use the provided graphql or REST binaries to publish the data.  Instructions for building and running the graphql server are provided in the [graphql/README.md](https://github.com/littlebunch/bfpd-rs/tree/master/graphql).  Instructions for the REST api are provided in the [restapi/README.md](https://github.com/littlebunch/bfpd-rs/tree/master/restapi).
