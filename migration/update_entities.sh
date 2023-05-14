#!/bin/bash

# Check if Docker and docker compose are installed
if ! command -v docker &> /dev/null || ! command -v docker compose &> /dev/null; then
  echo "Error: Docker or docker-compose command not found. Please install Docker and docker-compose."
  exit 1
fi

# Start the Docker container
echo "Starting Docker container..."
docker compose -f update-entities.yaml up -d

# Check if sea-orm-cli is installed
if ! command -v sea-orm-cli &> /dev/null; then
  cargo install sea-orm-cli
fi

echo "Waiting for container to stabilize"
sleep 1

# Run the sea-orm-cli command
echo "Running sea-orm-cli commands..."
sea-orm-cli migrate up -d . -u "postgres://entity:entity@localhost:5433/entity"
sea-orm-cli generate entity -u "postgres://entity:entity@localhost:5433/entity" -o ../src/entities

docker compose -f update-entities.yaml down
