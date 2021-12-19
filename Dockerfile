FROM lukemathwalker/cargo-chef:latest-rust-1.57 AS chef
WORKDIR /anabot

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /anabot/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release --bin anabot

# We do not need the Rust toolchain to run the binary!
FROM debian:buster-slim AS runtime
WORKDIR /anabot
COPY --from=builder /anabot/target/release/anabot /usr/local/bin
ENTRYPOINT ["/usr/local/bin/anabot"]
