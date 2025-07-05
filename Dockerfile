# ---- Chef ----
# Use the stable Rust 1.88 toolchain to support the 2024 edition.
FROM rust:1.88-bookworm as chef
WORKDIR /app
RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef
COPY . .
# Compute a recipe to determine the dependency graph.
RUN cargo chef prepare --recipe-path recipe.json

# ---- Planner ----
# This layer is cached as long as the recipe.json (dependency list) is the same.
FROM rust:1.88-bookworm as planner
WORKDIR /app
RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef
COPY --from=chef /app/recipe.json recipe.json
# Build only the dependencies, which will be cached.
RUN cargo chef cook --release --recipe-path recipe.json

# ---- Builder ----
# This layer is cached as long as the application source code (excluding dependencies) is the same.
FROM rust:1.88-bookworm as builder
WORKDIR /app
RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*
COPY . .
# Copy over the pre-built dependencies from the planner stage.
COPY --from=planner /app/target target
COPY --from=planner /usr/local/cargo /usr/local/cargo
# Build the application binary.
RUN cargo build --release --bin bingo

# ---- Runtime ----
# Use a minimal, non-root image for the final stage for better security.
FROM debian:bookworm-slim as runtime
WORKDIR /app

# Install ca-certificates, download grpc_health_probe, and create a non-root user.
# curl is installed temporarily and removed in the same layer to keep the image small.
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && curl -fL -o /usr/local/bin/grpc_health_probe https://github.com/grpc-ecosystem/grpc-health-probe/releases/download/v0.4.21/grpc_health_probe-linux-amd64 \
    && chmod +x /usr/local/bin/grpc_health_probe \
    && apt-get purge -y --auto-remove curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system nonroot \
    && useradd --system --gid nonroot nonroot

USER nonroot

# Copy the binary from the builder.
COPY --from=builder /app/target/release/bingo .

# Expose gRPC port
EXPOSE 50051

# Set environment variables for gRPC
ENV GRPC_LISTEN_ADDRESS=0.0.0.0:50051
ENV RUST_LOG=info

# Set the entrypoint.
ENTRYPOINT ["./bingo"]