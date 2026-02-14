# =============================================================================
# Stage 1: Builder - Build the Leptos SSR application
# =============================================================================
FROM rustlang/rust:nightly-alpine AS builder

# Install build dependencies
RUN apk update && \
    apk add --no-cache \
    bash \
    curl \
    npm \
    libc-dev \
    binaryen \
    openssl-dev \
    pkgconfig \
    sqlite \
    sqlite-dev

# Install sass for stylesheet compilation
RUN npm install -g sass

# Install sqlx-cli for preparing query metadata
RUN cargo install sqlx-cli --no-default-features --features sqlite

# Install cargo-leptos for building Leptos applications
RUN curl --proto '=https' --tlsv1.3 -LsSf \
    https://github.com/leptos-rs/cargo-leptos/releases/latest/download/cargo-leptos-installer.sh | sh

# Add WASM target for client-side hydration
RUN rustup target add wasm32-unknown-unknown

# Set working directory
WORKDIR /work

# Copy dependency manifests first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Copy source code and assets
COPY src ./src
COPY style ./style
COPY public ./public
COPY migrations ./migrations

# Generate SQLx offline query data on-the-fly
# This ensures queries are validated against the actual schema
ENV DATABASE_URL=sqlite:build.db
RUN sqlx database create && \
    sqlx migrate run && \
    cargo sqlx prepare --workspace -- --lib --features ssr && \
    rm build.db*

# Set SQLx to offline mode for the actual build
ENV SQLX_OFFLINE=true

# Build the application in release mode
# - Server binary compiled with SSR features
# - Client WASM bundle compiled with hydrate features
# - CSS processed through Tailwind and optimized
RUN cargo leptos build --release -vv

# =============================================================================
# Stage 2: Runner - Minimal runtime image
# =============================================================================
FROM rustlang/rust:nightly-alpine AS runner

# Install runtime dependencies including SQLite for sqlx-cli
RUN apk add --no-cache \
    ca-certificates \
    libgcc \
    sqlite \
    sqlite-dev

# Create app user for security
RUN addgroup -g 1000 appuser && \
    adduser -D -u 1000 -G appuser appuser

# Install sqlx-cli for database migrations
# It will be installed to /usr/local/cargo/bin/sqlx which is already in PATH
RUN cargo install sqlx-cli --no-default-features --features sqlite

WORKDIR /app

# Copy built artifacts from builder stage
COPY --from=builder /work/target/release/rustify-app /app/
COPY --from=builder /work/target/site /app/site
COPY --from=builder /work/Cargo.toml /app/
COPY --from=builder /work/migrations /app/migrations

# Copy entrypoint script
COPY docker-entrypoint.sh /app/
RUN chmod +x /app/docker-entrypoint.sh

# Create directory for SQLite database
RUN mkdir -p /app/data && \
    chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Configure environment variables
ENV RUST_LOG="info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_SITE_ROOT="./site"
ENV DATABASE_URL="sqlite:/app/data/splitify.db"

# Expose application port
EXPOSE 8080

# Health check endpoint (assumes /api/health exists or adjust as needed)
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080/ || exit 1

# Use entrypoint script to handle database setup
ENTRYPOINT ["/app/docker-entrypoint.sh"]
