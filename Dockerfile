# Define versions globally
ARG RUST_VERSION=1.95
ARG DEBIAN_VERSION=13-slim

# Stage 1: Base Image with development tools
FROM rust:${RUST_VERSION} AS base
RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake \
    libssl-dev \
    pkg-config \
    jq \
    && cargo install cargo-chef --locked \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user for the build process
RUN useradd -m rustuser \
    && mkdir -p /usr/local/cargo \
    && chown -R rustuser:rustuser /usr/local/cargo
USER rustuser
WORKDIR /home/rustuser/app

# Stage 2: Planner (Creates the dependency recipe)
FROM base AS planner
COPY --chown=rustuser:rustuser . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Cacher (Compiles only dependencies)
FROM base AS cacher
COPY --from=planner /home/rustuser/app/recipe.json recipe.json
# Use a cache mount for the cargo registry to speed up subsequent builds
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/home/rustuser/app/target \
    cargo chef cook --release --recipe-path recipe.json

# Stage 4: Builder (Compiles the actual application)
FROM base AS builder
COPY --from=planner /home/rustuser/app/recipe.json recipe.json
COPY --chown=rustuser:rustuser . .
# Copy pre-compiled dependencies
COPY --from=cacher /home/rustuser/app/target target
COPY --from=cacher /usr/local/cargo/registry /usr/local/cargo/registry

# Build and strip the binary to reduce size
RUN cargo build --release && \
    strip target/release/$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].targets[0].name')

# Stage 5: Core Runtime Base
FROM debian:${DEBIAN_VERSION} AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && apt-get clean && rm -rf /var/lib/apt/lists/*
WORKDIR /app

# Stage 6: SteamCMD Base
FROM runtime AS steamcmd
ENV DEBIAN_FRONTEND=noninteractive
ENV STEAMCMD_DIR="/home/steam/steamcmd"

RUN dpkg --add-architecture i386 && \
    apt-get update && \
    apt-get install -y --no-install-recommends \
    curl tar lib32gcc-s1 lib32stdc++6 procps && \
    rm -rf /var/lib/apt/lists/*

RUN useradd -m steam
USER steam
WORKDIR /home/steam

RUN mkdir -p ${STEAMCMD_DIR} && cd ${STEAMCMD_DIR} && \
    curl -sqL "https://steamcdn-a.akamaihd.net/client/installer/steamcmd_linux.tar.gz" | tar zxvf - && \
    ./steamcmd.sh +quit

ENV PATH="${PATH}:${STEAMCMD_DIR}"

# Stage 7: SteamCMD + Proton (For Windows-only game servers)
FROM steamcmd AS steamcmd-proton
USER root
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    wine \
    wine32 \
    wine64 \
    xvfb \
    libsdl2-2.0-0:i386 \
    && rm -rf /var/lib/apt/lists/*

USER steam
ENV PROTON_VERSION="GE-Proton10-34"
ENV PROTON_DIR="/home/steam/proton"
ENV STEAM_COMPAT_DATA_PATH="/home/steam/compatdata"

# Example download for GE-Proton (requires a direct URL to a release tarball)
RUN mkdir -p ${PROTON_DIR} ${STEAM_COMPAT_DATA_PATH} && \
    curl -sqL "https://github.com/GloriousEggroll/proton-ge-custom/releases/download/${PROTON_VERSION}/${PROTON_VERSION}.tar.gz" | tar -C ${PROTON_DIR} -zxvf -

ENV PATH="${PATH}:${PROTON_DIR}/${PROTON_VERSION}"

# Final Production Stage: Application + SteamCMD/Proton
FROM steamcmd-proton AS final
USER root
# Copy the compiled Rust binary from the builder
# Replace 'app-name' with your actual binary name if it differs
COPY --from=builder /home/rustuser/app/target/release/* /usr/local/bin/
USER steam
CMD ["bash"]