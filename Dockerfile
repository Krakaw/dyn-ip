FROM rust:1.62.0-bullseye as be_builder
WORKDIR /usr/src/dyn-ip

COPY ./Cargo.lock .
COPY ./Cargo.toml .
RUN mkdir src &&  echo "pub fn main() {}" >> src/main.rs
COPY public public
COPY src src
ARG CARGO_NET_GIT_FETCH_WITH_CLI=true
RUN cargo build --release


FROM debian:bullseye-slim
WORKDIR /opt/dyn-ip
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=be_builder /usr/src/dyn-ip/target/release/dyn-ip /usr/local/bin/dyn-ip
ENTRYPOINT ["dyn-ip"]
EXPOSE 8080
