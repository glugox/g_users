users
==================


Users service for Glugate as part of microservice architecture.


Dependencies:
-------------
* `rocket` web framework
* `diesel` ORM and Query Builder
* `serde` Serialization framework
* `validator` validation library
* `log` and `stderrlog` logging facade
* `clap` library for parsing command line arguments and subcommands
* `slug` Generating slugs from unicode strings
* `rust-crypto` Cryptographic algorithms used for user password management
* `rand` Random number generation
* `chrono` Dat and time library
* `dotenv` Loading from local .env file
* `jsonwebtoken` Authentication
* `assert_cli` integration testing


### Getting started

Install [nightly](https://www.rust-lang.org/en-US/install.html)
```sh
# install rustup
curl https://sh.rustup.rs -sSf | sh

rustup install nightly

# start postgresql and seed the database
psql -f init.sql
cargo install diesel_cli --no-default-features --features "postgres"
diesel migration run

cargo run
```

### Testing
Simply run:
```sh
cargo test
```
You can also check postman/newman. See `/tests` directory.

### How it works
`diesel` cli uses `.env` file.
Rocket sets database configuration from `.env` file.
Checkout Rocket's amazing [guide](https://rocket.rs/guide/)

### Features
By default random suffixes feature is enabled, so one could easily
create multiple articles with the same title. To disable it:

```sh
cargo run --no-default-features

```
