FROM rust:1.62.0 as builder


WORKDIR /app
RUN cargo new adana
WORKDIR /app/adana

COPY rust-toolchain.toml .

COPY ./Cargo.toml ./Cargo.lock ./

COPY .cargo/config .cargo/config

ENV RUSTFLAGS='-C link-arg=-s'

RUN cargo build --release 
RUN rm -rf ./src

COPY ./src/ ./src

RUN rm ./target/x86_64-unknown-linux-musl/release/deps/adana*

RUN cargo build --release 

FROM alpine

ENV RUST_LOG=info

VOLUME /root/.local/share

COPY --from=builder  /app/adana/target/x86_64-unknown-linux-musl/release/adana .
CMD [ "/adana" ]