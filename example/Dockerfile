FROM rust:1.100-trixie@sha256:1f65e0abcce9b449f9533df5994f2a62391059196c481a09b8e1b522dd6d75af AS build
WORKDIR /src
COPY . .
RUN cargo build --release -p example-function

FROM gcr.io/distroless/cc-debian13@sha256:dac90a25ffe03ebaace90cda886b081e61ab6cd77ac47eda51f7bcc15c320765 AS image
COPY --from=build /src/target/release/example-function /usr/local/bin/function
EXPOSE 9443
USER nonroot:nonroot
ENTRYPOINT ["function"]
