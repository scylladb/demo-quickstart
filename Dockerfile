FROM rust:1.72.1 as chef
WORKDIR app
RUN cargo install cargo-chef

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin scylladb-quick-demo-rs

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt install -y openssl

COPY --from=builder /app/target/release/scylladb-quick-demo-rs /app/scylladb-quick-demo-rs
COPY --from=builder /app/public /app/public
ENV ROCKET_ADDRESS=0.0.0.0

CMD ["/app/scylladb-quick-demo-rs"]