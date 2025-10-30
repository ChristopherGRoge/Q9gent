# Multi-stage build for smaller image
FROM rust:1.83-slim as builder

WORKDIR /app

# Install dependencies for building
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build release binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 q9gent

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/q9gent /usr/local/bin/q9gent

# Create session directory
RUN mkdir -p /app/sessions && \
    chown -R q9gent:q9gent /app

USER q9gent

# Expose default port
EXPOSE 8080

# Set default environment
ENV RUST_LOG=q9gent=info

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the binary
CMD ["q9gent", "--host", "0.0.0.0", "--port", "8080"]
