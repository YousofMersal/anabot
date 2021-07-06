FROM lukemathwalker/cargo-chef as planner
WORKDIR /data/anabot
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM lukemathwalker/cargo-chef as cacher
WORKDIR /data/anabot
COPY --from=planner /data/anabot/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust:latest as builder 
WORKDIR /data/anabot
COPY . . 
#Copy cache'd deps'
COPY --from=cacher /data/anabot/target /target
COPY --from=cacher $CARGO_HOME $CARGO_HOME
RUN cargo build --release --bin anabot

FROM debian:buster-slim as runtime
WORKDIR /data/anabot
COPY --from=builder /data/anabot/target/release/anabot /usr/local/bin
ENTRYPOINT ["/usr/local/bin/anabot"]
