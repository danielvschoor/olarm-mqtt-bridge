# syntax=docker/dockerfile:1

# Comments are provided throughout this file to help you get started.
# If you need more help, visit the Dockerfile reference guide at
# https://docs.docker.com/go/dockerfile-reference/

# Want to help us make this template better? Share your feedback here: https://forms.gle/ybq9Krt8jtBL3iCk7

ARG RUST_VERSION=1.89.0
ARG APP_NAME=olarm_mqtt_bridge

################################################################################
# Create a stage for building the application.

FROM rust:${RUST_VERSION}-bookworm AS build
ARG APP_NAME
WORKDIR /app

# Install host build dependencies.
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY src/ src/
COPY Cargo.toml Cargo.lock ./

RUN ls -la
# Build the application.
RUN cargo build --locked --release && \
cp ./target/release/$APP_NAME /bin/app

FROM debian:bookworm-slim AS final

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the executable from the "build" stage.
COPY --from=build /bin/app /app/app

# What the container should run when it is started.
CMD ["/app/app"]
