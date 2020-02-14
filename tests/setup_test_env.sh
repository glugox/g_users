#!/bin/bash

# Since we dont want tu run out test in the same database as production one,
# this script sets up *_test database with same migration scripts as the main database
# migration scripts.

source ../.env.test

(>&2 echo "Setting up testing environment")

if ! cargo build 2> build_err.log > build.log ; then
    echo "Cargo build failed"
    cat build_err.log
    cat build.log
    exit 1
fi
(>&2 echo "Done")


export GG_TEST_PORT=8000
export LOG_FILE=log.txt


echo "Database URL: $DATABASE_URL"


# Set up database
if ! diesel database setup --database-url "$DATABASE_URL" >> "$LOG_FILE"; then
    echo "Database setup failed"
    cat "$LOG_FILE"
    exit 1
fi

# make sure we have clean empty database
if ! diesel database reset --database-url "$DATABASE_URL" >> "$LOG_FILE"; then
    echo "Database reset failed"
    cat "$LOG_FILE"
    exit 1
fi

# run migration scripts, create tables, etc.
if ! diesel migrations run --database-url "$DATABASE_URL" >> "$LOG_FILE"; then
    echo "Database migrations failed"
    cat "$LOG_FILE"
    exit 1
fi



 echo "Sucessfully set up test environment. Created test database, and processed all migrations."