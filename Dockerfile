# Image untuk step Bitbucket Pipelines `tempa check --contract-only`.
FROM rust:1.85-alpine AS build
RUN apk add --no-cache musl-dev
WORKDIR /src
COPY . .
RUN cargo build --release -p tempa-cli --locked

FROM alpine:3.20
RUN apk add --no-cache git
COPY --from=build /src/target/release/tempa /usr/local/bin/tempa
ENTRYPOINT []
