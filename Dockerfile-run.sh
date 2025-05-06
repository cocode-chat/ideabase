#!/bin/bash

echo '-----> 1. Clean Docker Image'
docker image prune -f
# docker rmi -f $(docker images | grep "none" | awk '{print $3}')

echo '-----> 2. Build Docker Image'
docker build -f Dockerfile -t "cc/ideabase":latest .

echo '-----> 3. Clean Docker Container'
docker rm -f "ideabase-0"

echo "-----> 4. Run Docker Container"
docker-compose -f ".docker/Docker-compose.yaml" up -d

echo "-----> 5. Clear Dead Container"
docker container prune -f