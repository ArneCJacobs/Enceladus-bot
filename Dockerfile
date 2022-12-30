FROM rust
COPY ./rust-toolchain.toml .
RUN rustup toolchain install .
COPY . .
RUN cargo install --path .
CMD ["~/.cargo/bin/enceladus-bot"]
