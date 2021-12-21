FROM busybox:stable

ARG MACHINE

COPY out/${MACHINE}/flecs-backend /bin/flecs-backend
RUN chmod a+x /bin/flecs-backend

ENTRYPOINT [ "/bin/cp", "-pf", "/bin/flecs-backend", "/host/usr/bin/"  ]
