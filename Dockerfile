FROM rust:latest AS builder
WORKDIR /opt
COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc-debian11 AS runtime
COPY --from=builder /opt/target/release/miniature-parakeet /usr/local/bin/miniature-parakeet
ENV HTTP_PORT=8000
EXPOSE 8000
ENTRYPOINT ["/usr/local/bin/miniature-parakeet"]
