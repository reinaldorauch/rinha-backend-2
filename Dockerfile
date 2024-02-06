FROM rust:1.75 as build

RUN USER=root cargo new --bin rinha-backend-2

WORKDIR /rinha-backend-2

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src

RUN rm ./target/release/deps/rinha_backend_2*
RUN cargo build --release

FROM debian:bookworm-slim

COPY --from=build /rinha-backend-2/target/release/rinha-backend-2 .

EXPOSE 8080

CMD ["./rinha-backend-2"]
