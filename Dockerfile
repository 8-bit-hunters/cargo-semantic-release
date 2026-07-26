FROM rust:1-slim-trixie AS builder

WORKDIR /build
COPY . .

# libgit2-sys 0.18 requires libgit2 1.9, which debian:trixie ships (bookworm only has 1.5).
# Link against the system libgit2/OpenSSL via pkg-config instead of vendoring/compiling
# them from source.
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libgit2-dev libssl-dev \
    && rm -rf /var/lib/apt/lists/*

RUN cargo build --release

FROM debian:trixie-slim

# Dynamically linked against system libgit2/OpenSSL, so their runtime shared libs
# need to be present here too (the -dev packages from the builder stage aren't copied over).
RUN apt-get update \
    && apt-get install -y --no-install-recommends libgit2-1.9 libssl3t64 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/cargo-semantic-release /usr/local/bin/cargo-semantic-release

# libgit2 refuses to open repositories not owned by the current (container) user;
# the mounted GitHub Actions workspace is owned by the runner's host user instead.
# Written to /etc/gitconfig (system config), not ~/.gitconfig: GitHub Actions overrides
# $HOME to /github/home for Docker actions, which would shadow a user-level config.
RUN printf '[safe]\n\tdirectory = *\n' > /etc/gitconfig

ENTRYPOINT ["cargo-semantic-release", "semantic-release"]
