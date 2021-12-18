# Using the `rust-musl-builder` as base image, instead of 
# the official Rust toolchain
FROM ekidd/rust-musl-builder:1.51.0 AS chef
USER root
RUN cargo install cargo-chef
WORKDIR /anabot

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /anabot/recipe.json recipe.json
# Notice that we are specifying the --target flag!
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl --bin anabot

FROM alpine AS runtime
RUN addgroup -S myuser && adduser -S myuser -G myuser
COPY --from=builder /anabot/target/x86_64-unknown-linux-musl/release/anabot /usr/local/bin/
USER myuser
CMD ["/usr/local/bin/anabot"]
