FROM rust:1.56.1-slim as builder

WORKDIR /app
COPY Cargo* ./
COPY src src

RUN cargo build --release

# Running image:
FROM debian:stable-slim

COPY --from=builder /app/target/release/oak /usr/local/bin/oak
COPY poke.yml /

ENV ROCKET_ADDRESS="0.0.0.0"
ENV ROCKET_LOG_LEVEL="normal"

CMD ["oak", "--config", "/poke.yml"]
