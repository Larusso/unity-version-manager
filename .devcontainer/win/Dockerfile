FROM mcr.microsoft.com/devcontainers/rust:latest

RUN apt-get update && \
    apt-get install -y curl build-essential pkg-config libssl-dev mingw-w64

RUN rustup target add x86_64-pc-windows-gnu && \
    echo '[toolchain]\nchannel = "stable"\ntargets = ["x86_64-pc-windows-gnu"]' > /rust-toolchain.toml && \
    mkdir -p /.cargo && \
    echo '[build]\ntarget = "x86_64-pc-windows-gnu"\n\n[target.x86_64-pc-windows-gnu]\nlinker = "x86_64-w64-mingw32-gcc"' > /.cargo/config.toml
