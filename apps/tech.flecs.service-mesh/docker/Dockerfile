FROM debian:bookworm-slim

ARG ARCH

ADD fs/ /

ADD assets/zenoh-0.10.0-rc-${ARCH}.tar.gz /

ENTRYPOINT [ "/entrypoint.sh" ]
