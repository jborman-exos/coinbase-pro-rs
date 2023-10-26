FROM rust:1.67.1-slim-bullseye
RUN apt update && \
    apt upgrade -y && \
    apt install -y libssl-dev pkg-config
ENV CARGO_TARGET_DIR=/cargo_target/
