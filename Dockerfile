From rust:latest as build
WORKDIR /src
COPY . .
RUN cargo build -r

FROM ubuntu:rolling
WORKDIR /app
COPY --from=build /src/target/release/url_shortener /app/url_shortener
COPY --from=build /src/.env.default /app/.env.default
ENTRYPOINT ["/app/url_shortener", "serve"]