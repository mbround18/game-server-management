# Stage 1: Base Image
FROM rust:1.92 AS base

RUN apt-get update && apt-get install -y cmake \
    && cargo install cargo-chef --locked \
    && apt-get clean  \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m rustuser \
    && mkdir -p /usr/local/cargo \
    && chown -R rustuser:rustuser /usr/local/cargo

USER rustuser
WORKDIR /home/rustuser/app

# Stage 2: Planner
FROM base AS planner
# Copy the entire repository so that all workspace members (e.g., libs/*) are present.
COPY --chown=rustuser:rustuser . .

RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Cacher
FROM base AS cacher
WORKDIR /home/rustuser/app
COPY --chown=rustuser:rustuser --from=planner /home/rustuser/app/recipe.json .

RUN cargo chef cook --release --recipe-path recipe.json

# Stage 4: Builder
FROM base AS builder
WORKDIR /home/rustuser/app

COPY --chown=rustuser:rustuser . .

COPY --from=cacher /home/rustuser/app/target target
COPY --from=cacher /usr/local/cargo/registry /usr/local/cargo/registry

RUN cargo build --release

# Stage 5: Runtime
FROM debian:12-slim AS gh-runtime

RUN apt-get update && apt-get install -y libssl-dev curl jq unzip && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --chmod=0755 ./dist .

CMD ["bash"]


FROM debian:12-slim AS runtime

RUN apt-get update && apt-get install -y libssl-dev && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /home/rustuser/app/target/release/ .

CMD ["bash"]
