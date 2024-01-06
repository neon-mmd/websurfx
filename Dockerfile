FROM --platform=$BUILDPLATFORM rust:1.75.0-alpine3.18 AS chef
# We only pay the installation cost once,
# it will be cached from the second build onwards
RUN apk add --no-cache alpine-sdk musl-dev g++ make libcrypto3 libressl-dev upx perl build-base
RUN cargo install cargo-chef --locked

WORKDIR /app

FROM chef AS planner
COPY ./Cargo.toml ./Cargo.lock ./
RUN cargo chef prepare --recipe-path recipe.json

FROM --platform=$BUILDPLATFORM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Specify the cache type to use (memory, redis, hybrid, no-cache)
ARG CACHE=memory
ENV CACHE=${CACHE}
# Cook the dependencies
RUN export ARCH=$(uname -m) && \
  if [ "$CACHE" = "memory" ] ; then cargo chef cook --release --target=$ARCH-unknown-linux-musl --recipe-path recipe.json ; \
  else if [ "$CACHE" = "redis" ] ; then cargo chef cook --release --target=$ARCH-unknown-linux-musl --no-default-features --features redis-cache --recipe-path recipe.json ; \
  else if [ "$CACHE" = "hybrid" ] ; then cargo chef cook --release --target=$ARCH-unknown-linux-musl --features redis-cache --recipe-path recipe.json ; \
  else if [ "$CACHE" = "no-cache" ] ; then cargo chef cook --release --target=$ARCH-unknown-linux-musl --no-default-features --recipe-path recipe.json ; fi ; fi ; fi ; fi
# Copy the source code and public folder
COPY ./src ./src
COPY ./public ./public
# Build the application
RUN export ARCH=$(uname -m) && \
  if [ "$CACHE" = "memory" ] ; then cargo build --release --target=$ARCH-unknown-linux-musl ; \
  else if [ "$CACHE" = "redis" ] ; then cargo build --release --target=$ARCH-unknown-linux-musl --no-default-features --features redis-cache ; \
  else if [ "$CACHE" = "hybrid" ] ; then cargo build --release --target=$ARCH-unknown-linux-musl --features redis-cache ; \
  else if [ "$CACHE" = "no-cache" ] ; then cargo build --release --target=$ARCH-unknown-linux-musl --no-default-features ; fi ; fi ; fi ; fi
# Optimise binary size with UPX
RUN export ARCH=$(uname -m) \
  && upx --lzma --best /app/target/$ARCH-unknown-linux-musl/release/websurfx \
  && cp /app/target/$ARCH-unknown-linux-musl/release/websurfx /usr/local/bin/websurfx


FROM --platform=$BUILDPLATFORM scratch
COPY --from=builder /app/public/ /opt/websurfx/public/
VOLUME ["/etc/xdg/websurfx/"]
COPY --from=builder /usr/local/bin/websurfx /usr/local/bin/websurfx
CMD ["websurfx"]
