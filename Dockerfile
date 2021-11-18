FROM rust:1-slim-buster

RUN apt-get update \
  && export DEBIAN_FRONTEND=noninteractive \
  && apt-get install -y \
  cmake pkg-config libssl-dev git \
  build-essential clang libclang-dev \
  gcc curl vim

RUN rustup install nightly \
  && rustup target add wasm32-unknown-unknown --toolchain nightly \
  && rustup component add rust-src --toolchain nightly

ARG PROFILE=release
WORKDIR /dia

COPY . /dia

RUN cargo build --release \
  && cp -R /dia/target/release /usr/local/bin \
  && useradd -m -u 1000 -U -s /bin/sh -d /dia dia && \
  mkdir -p /dia/.local/share && \
  mkdir /data && \
  chown -R dia:dia /data 

USER dia
EXPOSE 30333 9933 9944 8070
VOLUME ["/data"]

CMD ["/usr/local/bin/"] 