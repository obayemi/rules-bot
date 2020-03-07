FROM rust:latest as BUILDER

WORKDIR /app
COPY . .
RUN cargo install --path .
RUN cargo install diesel_cli

# FROM alpine
# 
# COPY --from=builder /app/rules-bot ./
CMD ["rules-bot"]
