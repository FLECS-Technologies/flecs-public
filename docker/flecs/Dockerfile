FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get --yes --no-install-recommends install \
    ca-certificates docker.io git iptables procps && \
    apt-get --yes clean && \
    rm -rf /var/lib/apt/lists/*

RUN update-alternatives --set iptables /usr/sbin/iptables-legacy && \
    update-alternatives --set ip6tables /usr/sbin/ip6tables-legacy

ADD fs/ /

RUN ldconfig

ENTRYPOINT [ "/entrypoint.sh" ]
