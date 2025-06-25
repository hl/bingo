# Use Rust 2024 edition compatible image
FROM rust:1.82-slim-bookworm AS builder

# Install system dependencies for performance profiling and memory tracking
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    procps \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /usr/src/bingo

# Copy Cargo files for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY crates/bingo-core/Cargo.toml ./crates/bingo-core/
COPY crates/bingo-rete/Cargo.toml ./crates/bingo-rete/
COPY crates/bingo-api/Cargo.toml ./crates/bingo-api/

# Create dummy source files to build dependencies
RUN mkdir -p crates/bingo-core/src crates/bingo-rete/src crates/bingo-api/src && \
    echo "fn main() {}" > crates/bingo-api/src/main.rs && \
    echo "// dummy" > crates/bingo-core/src/lib.rs && \
    echo "// dummy" > crates/bingo-rete/src/lib.rs && \
    echo "// dummy" > crates/bingo-api/src/lib.rs

# Build dependencies (cached layer)
RUN cargo build --release

# Remove dummy source files
RUN rm -rf crates/*/src

# Copy actual source code
COPY crates/ ./crates/

# Build the application
RUN cargo build --release --bin bingo

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    procps \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN useradd -r -s /bin/false bingo

# Copy binary from builder stage
COPY --from=builder /usr/src/bingo/target/release/bingo /usr/local/bin/bingo

# Set proper ownership
RUN chown bingo:bingo /usr/local/bin/bingo

# Switch to non-root user
USER bingo

# Expose port (default for private network deployment)
EXPOSE 8080

# Health check endpoint
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Set environment for production
ENV RUST_LOG=info
ENV BINGO_HOST=0.0.0.0
ENV BINGO_PORT=8080

# Run the binary
CMD ["bingo"]