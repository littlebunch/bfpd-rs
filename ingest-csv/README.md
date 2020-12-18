# ingest-csv

A command line utility for parsing [USDA Branded Food Products](https://fdc.nal.usda.gov) CSV files and loading into a database.  The database can be either [PostgreSQL](https://www.postgresql.org) or [MariaDB](https://www.mariadb.com).  

## What's here
[./src/main.rs](https://github.com/littlebunch/bfpd-rs/blob/master/ingest-csv/src/ingest-csv.rs) -- cli utility for importing the USDA csv files into the database  
[./src/clap.yml](https://github.com/littlebunch/bfpd-rs/blob/master/ingest-csv/src/clap.yml) -- configuration for CLI parsing

## How to Build

### Step 1: Set-up your environment

If you haven't already, install the Rust [toolchain](https://www.rust-lang.org/tools/install) in your work environment.  It's assumed you have the database loaded in either PostgreSQL or MariaDB.  Refer to the [README](https://github.com/littlebunch/bfpd-rs) for how to do this.

### Step 2: Clone this repo

```bash
git clone git@github.com:littlebunch/bfpd-rs.git
```

### Step 3: Build the binary

If you are using PostgreSQL:

```bash
cd ./ingest-csv
cargo build --release --features postgres
```

If you are using MariaDB

```bash
cd ./ingest-csv
cargo build --release --features maria
```

This will create the ingest-csv program in the top-level ./target/release directory.  

### Step 4: Run the ingest-csv utility

You need to set the database URL environment variable.  

```bash
DATABASE_URL=postgres://user:password@localhost/bfpd
```

or  

```bash
DATABASE_URL=mysql://user:password@localhost/bfpd
```

It generally makes sense to put it in an .env file in the top-level path of your workspace.  Then, point the program to the directory containing the CSV files:

```bash
./target/release/ingest-cvs -p /path/to/csv/
```

The load takes about 3-10 minutes depending on your hardware.  Note:  you need to set a DATABASE_URL variable as described in Step 2 below before running the ingest-csv program.
