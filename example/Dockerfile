FROM rust:1.99-trixie@sha256:e259494b14672950a909b4c54527e19d29cae6ebf980672114c8bb85e5594e4a AS build
WORKDIR /src
COPY . .
RUN cargo build --release -p example-function

FROM gcr.io/distroless/cc-debian13@sha256:159783207c2cd44c2aa5715961d13c8612368ac9bd450f887e3f08fc8ea461e3 AS image
COPY --from=build /src/target/release/example-function /usr/local/bin/function
EXPOSE 9443
USER nonroot:nonroot
ENTRYPOINT ["function"]
