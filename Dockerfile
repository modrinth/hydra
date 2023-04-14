FROM rust:1.68.2 as build
ENV PKG_CONFIG_ALLOW_CROSS=1

WORKDIR /usr/src/hydra
COPY . .
RUN cargo build --release


FROM debian:bullseye-slim

RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

RUN update-ca-certificates

COPY --from=build /usr/src/hydra/target/release/hydra /hydra/hydra
COPY --from=build /usr/src/hydra/templates /hydra/templates
COPY --from=build /usr/src/hydra/assets /hydra/assets
WORKDIR /hydra

CMD /hydra/hydra
w