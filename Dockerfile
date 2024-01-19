# Use the official Rust image as the base image for the builder stage
FROM rust:slim-bookworm as builder

# Install trunk
RUN \
    --mount=type=cache,target=/usr/local/cargo/registry/cache \
    --mount=type=cache,target=/usr/local/cargo/registry/index \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    cargo install trunk
RUN rustup target add wasm32-unknown-unknown

# Set the working directory
WORKDIR /usr/src/app

# Copy the actual source code
COPY Cargo.toml Cargo.lock ./
COPY taxy taxy
COPY taxy-api taxy-api
COPY taxy-webui taxy-webui

# Build the web UI
WORKDIR /usr/src/app/taxy-webui
RUN \
    --mount=type=cache,target=/usr/local/cargo/registry/cache \
    --mount=type=cache,target=/usr/local/cargo/registry/index \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/root/.cache/trunk \
    trunk build --release
WORKDIR /usr/src/app

# Build the Rust project
RUN \
    --mount=type=cache,target=/usr/local/cargo/registry/cache \
    --mount=type=cache,target=/usr/local/cargo/registry/index \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/src/app/target \
    cargo install --path taxy

# Prepare the final image
FROM debian:bookworm-slim as runtime

# Install dependencies for the Rust binary
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /app

# Copy the Rust binary from the builder stage
COPY --from=builder /usr/local/cargo/bin/taxy /usr/local/bin

# Set the entrypoint to run the Rust binary
ENTRYPOINT ["taxy", "start", "--webui", "0.0.0.0:46492"]
