FROM debian:buster-slim

ARG MACHINE

COPY [ \
    "docker/libzenohc.so", \
    "/usr/lib/x86_64-linux-gnu/" ]

COPY [ \
    "docker/mqtt_bridge", \
    "/usr/bin/" ]

ENTRYPOINT [ \
    "/usr/bin/mqtt_bridge" ]
