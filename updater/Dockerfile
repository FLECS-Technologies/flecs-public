FROM busybox:stable

ARG MACHINE
ARG ARCH
ARG VERSION

COPY [ \
    "out/${MACHINE}/opt/flecs/pkg/flecs.deb.tar.gz", \
    "out/${MACHINE}/opt/flecs/pkg/flecs-updater.deb.tar.gz", \
    "/pkg/" ]

ENTRYPOINT [ \
    "/bin/cp", "-f", \
    "/pkg/flecs.deb.tar.gz", \
    "/pkg/flecs-updater.deb.tar.gz", \
    "-t", "/host/usr/bin/" ]
