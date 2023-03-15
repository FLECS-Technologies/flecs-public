FROM debian:bullseye-slim

ARG ARCH

ADD assets/zenoh-0.7.0-rc-${ARCH}.tar.gz /

COPY assets/entrypoint.sh /

ENTRYPOINT [ "/entrypoint.sh" ]
