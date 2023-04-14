FROM rust as builder
COPY ./rust-toolchain.toml .
RUN rustup toolchain install "nightly-2022-12-17"
COPY . .
RUN cargo install --path .

# FROM alpine:3.14
FROM debian:buster-slim
WORKDIR /usr/local/bin/
COPY --from=builder /usr/local/cargo/bin/enceladus-bot ./
CMD ["./enceladus-bot"]
