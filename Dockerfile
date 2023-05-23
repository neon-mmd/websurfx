FROM rust:latest AS chef
# We only pay the installation cost once,
# it will be cached from the second build onwards
RUN cargo install cargo-chef

WORKDIR /app
FROM chef AS planner
COPY . . 
RUN cargo chef prepare --recipe-path recipe.json

# Builder
FROM chef AS builder

WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY . .
RUN cargo install --path .

# We do not need the Rust toolchain to run the binary!
FROM gcr.io/distroless/cc-debian11
WORKDIR /app
COPY --from=builder ./public/ ./public/
COPY --from=builder ./websurfx/ ./websurfx/
COPY --from=builder /usr/local/cargo/bin/* /usr/local/bin/
CMD ["websurfx"]
