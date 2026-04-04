FROM rust:1.88-slim-bookworm

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy Cargo files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs to build dependencies
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Copy actual source code
COPY src ./src

# Force rebuild by touching main.rs (avoids cache issue with dummy build)
RUN touch src/main.rs && cargo build --release

EXPOSE 3001

CMD ["./target/release/dioxus-storage-backend"]
