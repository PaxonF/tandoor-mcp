# Multi-stage build for mcp-tandoor server
# Build stage
# Pin to bookworm so the binary's glibc matches the distroless cc-debian12 runtime
FROM rust:1.89-slim-bookworm AS builder

# Install system dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml ./

# Build dependencies against stub sources so this layer stays cached until
# Cargo.toml changes, instead of recompiling every crate on each source edit
RUN mkdir src \
    && echo 'fn main() {}' > src/main.rs \
    && touch src/lib.rs \
    && cargo build --release \
    && rm -rf src

# Copy source code
COPY src ./src

# Build the application in release mode (touch so cargo sees the real sources as newer)
RUN touch src/main.rs src/lib.rs && cargo build --release

# Runtime base using distroless
FROM gcr.io/distroless/cc-debian12:latest AS base

# Copy the binary from builder stage
COPY --from=builder /app/target/release/mcp-tandoor /usr/local/bin/mcp-tandoor

# Set environment variables with defaults
ENV TANDOOR_BASE_URL=http://localhost:8080
ENV TANDOOR_USERNAME=admin
ENV BIND_ADDR=0.0.0.0:3001
ENV RUST_LOG=info

# Expose the port
EXPOSE 3001

# Run the binary
ENTRYPOINT ["/usr/local/bin/mcp-tandoor"]

# Home Assistant app variant (build with --target ha). Supervisor writes
# /data/options.json as root with mode 0600, so this variant must run as root
# to read the settings from the app's Configuration tab.
FROM base AS ha
USER 0:0

# Default image: run as a non-root user for security
FROM base AS runtime
USER 1000:1000
