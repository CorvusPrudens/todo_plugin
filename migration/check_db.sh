#!/bin/bash


# Load variables from .env file
if [ -f ../.env ]; then
  export $(grep -v '^#' .env | xargs)
else
  echo "Error: .env file not found in the root directory."
  exit 1
fi

if [ -z "$DB_URL" ] || [ -z "$DB_NAME" ]; then
  echo "Error: DB_URL or DB_NAME is not set in the .env file."
  exit 1
fi

# Check if the PostgreSQL command line utilities are installed
if ! command -v psql &> /dev/null; then
  echo "Error: psql command not found. Please install PostgreSQL command line utilities."
  exit 1
fi

# Check if the database exists
DB_EXISTS=$(psql "${DB_URL}/postgres" -tAc "SELECT 1 FROM pg_database WHERE datname='${DB_NAME}'")

if [ "$DB_EXISTS" = "1" ]; then
  echo "Database '${DB_NAME}' already exists."
else
  # Create the database
  echo "Creating database '${DB_NAME}'..."
  psql "${DB_URL}/postgres" -c "CREATE DATABASE ${DB_NAME};"
  echo "Database '${DB_NAME}' created."
fi
