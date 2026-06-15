FROM rust:1.86-alpine3.21

WORKDIR /workspace

RUN apk add --no-cache \
        clang \
        lld \
        make \
        musl-dev \
        pkgconf \
        protobuf \
        protobuf-dev \
    && adduser -D -u 10001 icpx

COPY Cargo.toml Cargo.lock ./
COPY programs ./programs
COPY docs ./docs

RUN cargo fetch
RUN chown -R icpx:icpx /workspace /usr/local/cargo

USER icpx

CMD ["cargo", "test", "--workspace"]
