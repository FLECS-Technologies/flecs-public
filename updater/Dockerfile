FROM busybox:stable

ARG MACHINE

COPY [ \
    "usr/bin/flecs-service.deb.tar.gz", "usr/bin/flecs", \
    "usr/bin/flecs-backend.deb.tar.gz", \
    "usr/bin/flecs-updater.deb.tar.gz", "usr/bin/flecs-updater", \
    "/bin/" ]
RUN chmod a+x /bin/flecs*

ENTRYPOINT [ \
    "/bin/cp", "-f", \
    "/bin/flecs-service.deb.tar.gz", "/bin/flecs", \
    "/bin/flecs-backend.deb.tar.gz", \
    "/bin/flecs-updater.deb.tar.gz", "/bin/flecs-updater", \
    "-t", "/host/usr/bin/" ]
