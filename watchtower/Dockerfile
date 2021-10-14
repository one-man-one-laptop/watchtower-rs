FROM rust:1.49 as builder
WORKDIR /usr/src/watchtower
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
RUN apt-get update && apt-get install -y libssl-dev && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/watchtower /usr/local/bin/watchtower
CMD ["watchtower"]