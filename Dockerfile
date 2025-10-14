FROM rust:1.90-slim AS be_builder
WORKDIR /usr/src/dyn-ip

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*
COPY ./Cargo.lock .
COPY ./Cargo.toml .
RUN mkdir src &&  echo "pub fn main() {}" >> src/main.rs
COPY public public
COPY src src
ARG CARGO_NET_GIT_FETCH_WITH_CLI=true
ENV OPENSSL_STATIC=0
ENV OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu
ENV OPENSSL_INCLUDE_DIR=/usr/include/openssl
RUN cargo build --release


FROM debian:trixie-slim
WORKDIR /opt/dyn-ip
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=be_builder /usr/src/dyn-ip/target/release/dyn-ip /usr/local/bin/dyn-ip
ENTRYPOINT ["dyn-ip"]
EXPOSE 8080
