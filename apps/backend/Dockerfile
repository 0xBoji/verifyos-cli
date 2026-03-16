FROM rust:latest AS builder

RUN apt-get update && apt-get install -y musl-tools
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /app
COPY . .

# Build with MUSL for a static binary
RUN cargo build --release --target x86_64-unknown-linux-musl --manifest-path apps/backend/Cargo.toml

FROM alpine:latest
RUN apk add --no-cache ca-certificates

COPY --from=builder /app/apps/backend/target/x86_64-unknown-linux-musl/release/verifyos-backend /usr/local/bin/verifyos-backend

ENV RUST_LOG=info
EXPOSE 7070

CMD ["verifyos-backend"]
