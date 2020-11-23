FROM rust:alpine3.12 as builder
WORKDIR /usr/src/waitfree-rust
COPY . . 
RUN apk add --no-cache musl-dev
RUN cargo build --release

FROM alpine:3.12
COPY --from=builder /usr/src/waitfree-rust/target/release/waitfree-rust /usr/local/bin/waitfree-rust
CMD ["waitfree-rust"]

