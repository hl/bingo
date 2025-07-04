# ---- Chef ----
# Use the stable Rust 1.88 toolchain to support the 2024 edition.
FROM rust:1.88-bookworm as chef
WORKDIR /app
RUN cargo install cargo-chef
COPY . .
# Compute a recipe to determine the dependency graph.
RUN cargo chef prepare --recipe-path recipe.json

# ---- Planner ----
# This layer is cached as long as the recipe.json (dependency list) is the same.
FROM rust:1.88-bookworm as planner
WORKDIR /app
COPY --from=chef /app/recipe.json recipe.json
# Build only the dependencies, which will be cached.
RUN cargo chef cook --release --recipe-path recipe.json

# ---- Builder ----
# This layer is cached as long as the application source code (excluding dependencies) is the same.
FROM rust:1.88-bookworm as builder
WORKDIR /app
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
RUN groupadd --system nonroot && useradd --system --gid nonroot nonroot
USER nonroot

# Copy the binary from the builder.
COPY --from=builder /app/target/release/bingo .

# Set the entrypoint.
ENTRYPOINT ["./bingo"]