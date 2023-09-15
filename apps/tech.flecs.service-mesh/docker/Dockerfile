FROM debian:bookworm-slim

ARG ARCH

ADD fs/ /

ADD assets/zenoh-0.6.0-beta.1-${ARCH}.tar.gz /

ENTRYPOINT [ "/entrypoint.sh" ]
