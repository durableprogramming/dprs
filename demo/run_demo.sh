#!/bin/sh

# Exit immediately if a command exits with a non-zero status.
set -e

# Get the directory of the script
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"

# Docker Compose file path
COMPOSE_FILE="$SCRIPT_DIR/docker-compose.yml"

# VHS tape file path
TAPE_FILE="$SCRIPT_DIR/demo.tape"

# Variable to store the Docker Compose command
DOCKER_COMPOSE_CMD=""

export PS1="> "


# Function to ensure Docker and Docker Compose are available
check_dependencies() {
    if ! command -v docker &> /dev/null; then
        echo "Error: docker command not found. Please install Docker." >&2
        exit 1
    fi

    if command -v docker-compose &> /dev/null; then
        echo "Using 'docker-compose' (V1)"
        DOCKER_COMPOSE_CMD="docker-compose"
    elif docker compose version &> /dev/null; then
        echo "Using 'docker compose' (V2)"
        DOCKER_COMPOSE_CMD="docker compose"
    else
        echo "Error: docker-compose (or docker compose) command not found. Please install Docker Compose." >&2
        exit 1
    fi

    if ! command -v vhs &> /dev/null; then
        echo "Error: vhs command not found. Please install VHS (https://github.com/charmbracelet/vhs)." >&2
        exit 1
    fi
}

# Function to clean up Docker Compose environment
cleanup() {
    echo "" # Newline for cleaner output
    echo "Shutting down demo environment..."
    if [ -n "$DOCKER_COMPOSE_CMD" ]; then # Check if DOCKER_COMPOSE_CMD was set
        if [ "$DOCKER_COMPOSE_CMD" == "docker compose" ]; then
            docker compose -f "$COMPOSE_FILE" down --remove-orphans --volumes
        else
            docker-compose -f "$COMPOSE_FILE" down --remove-orphans --volumes
        fi
        echo "Demo environment shut down and volumes removed."
    else
        echo "Docker Compose command was not determined; manual cleanup might be needed if services were started."
    fi
}

# Trap EXIT signal to ensure cleanup runs
trap cleanup EXIT

# Check for required tools
check_dependencies

echo "Starting demo environment with Docker Compose..."
# Use the determined Docker Compose command
if [ "$DOCKER_COMPOSE_CMD" == "docker compose" ]; then
    docker compose -f "$COMPOSE_FILE" up -d
else
    docker-compose -f "$COMPOSE_FILE" up -d
fi

sleep 5

echo "Recording demo with VHS..."
# VHS will output to demo/demo.gif as per demo.tape configuration
vhs "$TAPE_FILE"

echo "Demo recording complete. GIF should be at $SCRIPT_DIR/demo.gif"

# Explicitly exit successfully if we reach here. Cleanup will still run.
exit 0
