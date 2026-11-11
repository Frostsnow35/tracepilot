# 构建阶段
FROM rust:1.83-slim AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock* ./
COPY crates ./crates
COPY migrations ./migrations

RUN cargo build --release --locked || cargo build --release

# 运行阶段
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends libpq5 ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/tracepilot-api /usr/local/bin/tracepilot-api

EXPOSE 8000
CMD ["tracepilot-api"]
