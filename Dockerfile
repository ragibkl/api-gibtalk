FROM rust:slim-bookworm as builder

WORKDIR /usr/src/api-gibtalk
COPY src src
COPY Cargo.toml Cargo.lock .
RUN cargo build --release


FROM debian:bookworm-slim as runtime

WORKDIR /opt/api-gibtalk
COPY media media
COPY --from=builder /usr/src/api-gibtalk/target/release/api-gibtalk /usr/local/bin/api-gibtalk
CMD ["api-gibtalk"]
