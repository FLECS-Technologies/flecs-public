FROM debian:buster-slim

ARG MACHINE

COPY [ \
    "docker/libzenohc.so", \
    "/usr/lib/" ]

COPY [ \
    "docker/mqtt_bridge", \
    "/usr/bin/" ]

ENTRYPOINT [ \
    "/usr/bin/mqtt_bridge" ]
