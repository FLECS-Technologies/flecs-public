FROM busybox:stable

ARG MACHINE

COPY [ \
    "usr/bin/flecs-service.deb.shar", "usr/bin/flecsd", \
    "usr/bin/flecs-backend.deb.shar", "usr/bin/flecs-backend", \
    "usr/bin/flecs-updater.deb.shar", "/bin/" ]
RUN chmod a+x /bin/flecs*

ENTRYPOINT [ \
    "/bin/cp", "-f", \
    "/bin/flecs-service.deb.shar", "/bin/flecsd", \
    "/bin/flecs-backend.deb.shar", "/bin/flecs-backend", \
    "/bin/flecs-updater.deb.shar", "-t", "/host/usr/bin/" ]
