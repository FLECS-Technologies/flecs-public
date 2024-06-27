FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get --yes --no-install-recommends install \
    ca-certificates docker.io git iptables procps nginx && \
    apt-get --yes clean && \
    rm -rf /var/lib/apt/lists/*

RUN update-alternatives --set iptables /usr/sbin/iptables-legacy && \
    update-alternatives --set ip6tables /usr/sbin/ip6tables-legacy

ADD fs/ /

RUN echo "/usr/local/lib" >/etc/ld.so.conf.d/00-flecs.conf && ldconfig

ENTRYPOINT [ "/entrypoint.sh" ]
