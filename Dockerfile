FROM rust:1-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin nidaime-takohachi

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/nidaime-takohachi /app/nidaime-takohachi
CMD ["/app/nidaime-takohachi"]
