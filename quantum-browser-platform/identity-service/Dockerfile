# Multi-stage build for identity_service
FROM rust:1.72 as builder
WORKDIR /usr/src/app
COPY . .
WORKDIR /usr/src/app
RUN cd ./ && cargo install --path . --locked --root /usr/local

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates postgresql-client && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/bin/identity_service /usr/local/bin/identity_service
# copy entrypoint
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
ENV RUST_LOG=info
EXPOSE 4608
ENTRYPOINT ["/entrypoint.sh"]
