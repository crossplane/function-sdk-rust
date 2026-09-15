FROM rust:1.98-bookworm@sha256:9a73a5088750b4c95158ab26629c854c3d6fc4b173cb7bc8079ad252d8ed7bfa AS build
WORKDIR /src
COPY . .
RUN cargo build --release -p example-function

FROM gcr.io/distroless/cc-debian12@sha256:e5d81ddde149641e2a9ba55be4545bc125c67de07508b03ba4c22e6eb0ded5aa AS image
COPY --from=build /src/target/release/example-function /usr/local/bin/function
EXPOSE 9443
USER nonroot:nonroot
ENTRYPOINT ["function"]
