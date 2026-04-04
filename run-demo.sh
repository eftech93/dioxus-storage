#!/bin/bash

# Dioxus Storage Sync Demo - One Command Runner
# Usage: ./run-demo.sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Dioxus Storage Sync Demo${NC}"
echo -e "${BLUE}========================================${NC}"

# Check if docker is running
if ! docker info > /dev/null 2>&1; then
    echo -e "${RED}Error: Docker is not running${NC}"
    exit 1
fi

# Function to cleanup on exit
cleanup() {
    echo -e "\n${YELLOW}Shutting down...${NC}"
    cd examples/sync-demo/backend && docker-compose down 2>/dev/null || true
    exit 0
}
trap cleanup INT TERM

# Start backend
echo -e "\n${GREEN}▶ Starting backend (MongoDB + API)...${NC}"
cd examples/sync-demo/backend

# Rebuild and start services
docker-compose down 2>/dev/null || true
docker-compose up --build -d

# Wait for API to be healthy
echo -e "${YELLOW}  Waiting for API to be ready...${NC}"
for i in {1..30}; do
    if curl -s http://localhost:3001/api/health > /dev/null 2>&1; then
        echo -e "${GREEN}  ✓ API is ready on http://localhost:3001${NC}"
        break
    fi
    sleep 1
    echo -n "."
done

# Verify health
echo -e "\n${BLUE}  Health check:${NC}"
curl -s http://localhost:3001/api/health | jq . 2>/dev/null || curl -s http://localhost:3001/api/health

# Test API
echo -e "\n${BLUE}  Testing API:${NC}"
PRODUCTS=$(curl -s "http://localhost:3001/api/products?page=1&per_page=2" | jq '.total' 2>/dev/null || echo "?")
echo -e "  Products in database: ${GREEN}${PRODUCTS}${NC}"

cd ..

# Check if dx is installed
if ! command -v dx &> /dev/null; then
    echo -e "\n${YELLOW}⚠ dioxus-cli not found. Installing...${NC}"
    cargo install dioxus-cli
fi

# Start frontend
echo -e "\n${GREEN}▶ Starting frontend (Dioxus app)...${NC}"

echo -e "${BLUE}========================================${NC}"
echo -e "${GREEN}  All services starting!${NC}"
echo -e "${BLUE}========================================${NC}"
echo -e "  ${YELLOW}Backend API:${NC} http://localhost:3001"
echo -e "  ${YELLOW}Frontend:${NC}    http://localhost:8080 (or check dx output)"
echo -e "${BLUE}========================================${NC}"
echo -e "  Press Ctrl+C to stop all services"
echo -e "${BLUE}========================================${NC}\n"

dx serve --platform web
