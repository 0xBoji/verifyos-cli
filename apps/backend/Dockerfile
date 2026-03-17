FROM rust:1-bookworm AS builder

# Install system dependencies if needed (e.g. openssl, although it seems we don't need it yet)
# RUN apt-get update && apt-get install -y pkg-config libssl-dev

WORKDIR /app
COPY . .

# Build for the native architecture or specified platform
RUN cargo build --release --manifest-path apps/backend/Cargo.toml && \
    mkdir -p /app/bin && \
    find /app -name verifyos-backend -type f -executable -exec cp {} /app/bin/verifyos-backend \;

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the binary from the staging bin folder
COPY --from=builder /app/bin/verifyos-backend /usr/local/bin/verifyos-backend

ENV RUST_LOG=info
EXPOSE 7070

CMD ["verifyos-backend"]
