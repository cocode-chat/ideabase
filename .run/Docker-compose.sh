#!/bin/bash
echo '-----> 1. Clean Docker Image'
docker image prune -f

echo '-----> 2. Build Docker Image'
docker build -f Dockerfile -t "cc/ideabase":latest .

echo '-----> 3. Clean Docker Container'
docker rm -f "ideabase"

echo "-----> 4. Run Docker Container"
docker-compose -f ".run/Docker-compose.yaml" up -d

echo "-----> 5. Clear Dead Container"
docker container prune -f