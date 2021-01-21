# syntax=docker/dockerfile:experimental
FROM rust:latest as BUILDER                                                                   

WORKDIR /app
COPY . .
#RUN --mount=type=cache,target=/root/.cargo cargo build-
#RUN --mount=type=cache,target=./target/release/deps cargo build --release
RUN --mount=type=cache,target=./target cargo install --path . --target-dir target

CMD ["rules-bot"]

FROM rust as runtime

RUN cargo install diesel_cli

WORKDIR app
COPY ./migrations ./
COPY --from=builder /usr/local/cargo/bin/rules-bot /usr/local/bin

CMD ["/usr/local/bin/rules-bot"]
