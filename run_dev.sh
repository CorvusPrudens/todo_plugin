#!/bin/bash

# Check if Docker and docker compose are installed
if ! command -v docker &> /dev/null || ! command -v docker compose &> /dev/null; then
  echo "Error: Docker or docker-compose command not found. Please install Docker and the docker compose utility."
  exit 1
fi

# Start the Docker container
echo "Starting Docker container..."
docker compose -f docker-compose.dev.yaml up -d

echo "Waiting for container to stabilize"
sleep 1

echo "Running program"
DB_URL="postgres://test:test@localhost:5433" DB_NAME="test" cargo run --release ; docker compose -f docker-compose.dev.yaml down
