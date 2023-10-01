FROM rust:latest AS chef
# We only pay the installation cost once,
# it will be cached from the second build onwards
RUN cargo install cargo-chef --locked

WORKDIR /app

FROM chef AS planner
COPY . . 
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
# Uncomment the line below if you want to use the `hybrid` caching feature.
# RUN cargo chef cook --release --features redis-cache --recipe-path recipe.json
# Comment the line below if you don't want to use the `In-Memory` caching feature.
RUN cargo chef cook --release --recipe-path recipe.json
# Uncomment the line below if you want to use the `no cache` feature.
# RUN cargo chef cook --release --no-default-features --recipe-path recipe.json
# Uncomment the line below if you want to use the `redis` caching feature.
# RUN cargo chef cook --release --no-default-features --features redis-cache --recipe-path recipe.json

# Build application
COPY . .
# Uncomment the line below if you want to use the `hybrid` caching feature.
# RUN cargo install --path . --features redis-cache
# Comment the line below if you don't want to use the `In-Memory` caching feature.
RUN cargo install --path .
# Uncomment the line below if you want to use the `no cache` feature.
# RUN cargo install --path . --no-default-features 
# Uncomment the line below if you want to use the `redis` caching feature.
# RUN cargo install --path . --no-default-features --features redis-cache

# We do not need the Rust toolchain to run the binary!
FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/public/ /opt/websurfx/public/
VOLUME ["/etc/xdg/websurfx/"]
COPY --from=builder /usr/local/cargo/bin/* /usr/local/bin/
CMD ["websurfx"]
