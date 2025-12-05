#!/bin/bash
# Copyright 2021-2023 FLECS Technologies GmbH
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
# http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

PATH=/sbin:/usr/sbin:/bin:/usr/bin:/usr/local/sbin:/usr/local/bin
FLOXY_CONTAINER_NAME=flecs-floxy
# TODO: Change tag to latest before release
FLOXY_IMAGE=flecspublic.azurecr.io/flecs/floxy:debug

create_network() {
  NETWORK=flecs
  # check if we have created network before
  GATEWAY=`docker network inspect --format "{{range .IPAM.Config}}{{.Gateway}}{{end}}" "$NETWORK" 2>/dev/null`

  # if network does not exist, create it
  if [ -z "${GATEWAY}" ]; then
    # list all in-use IP addresses
    if ifconfig -a >/dev/null 2>&1; then
      IPS=`ifconfig -a | sed -n -E 's/^[[:space:]]+inet ([0-9\.]+).+$/\1/p'`
    elif ip addr >/dev/null 2>&1; then
      IPS=`ip addr -a | sed -n -E 's/^[[:space:]]+inet ([0-9\.]+).+$/\1/p'`
    else
      echo "Warning: Cannot determine in-use IP addresses" 1>&2
    fi
    # try subnets 172.21.0.0/16 --> 172.31.0.0/16
    SUBNETS=(21 22 23 24 25 26 27 28 29 30 31)
    for SUBNET in "${SUBNETS[@]}"; do
      # skip subnets that overlap with in-use IP addresses
      SKIP_SUBNET=
      for IP in ${IPS}; do
        if [[ ${IP} == 172.${SUBNET}.* ]]; then
          echo "${IP} collides with subnet 172.${SUBNET}.0.0/16 -- skipping"
          SKIP_SUBNET="true"
        fi
      done
      if [ ! -z "${SKIP_SUBNET}" ]; then
        continue
      fi
      # try to create network as Docker bridge network
      if docker network create --driver bridge --subnet "172.${SUBNET}.0.0/16" --gateway "172.${SUBNET}.0.1" "$NETWORK" >/dev/null 2>&1; then
        GATEWAY="172.${SUBNET}.0.1"
        echo "Created network '$NETWORK'"
        break;
      fi
    done
  else
    echo "Reusing existing network '$NETWORK'"
  fi

  if [ -z "${GATEWAY}" ]; then
    echo "Network '$NETWORK' does not exist and could not be created" 2>&1
    exit 1
  fi
  local __resultvar=$1
  local __value="${GATEWAY}"
  eval $__resultvar="'$__value'"
}

determine_free_http_port() {
    HTTP_PORTS=(80 8080 8000 none)
    HTTP_PORTS_HEX=(0050 1F90 1F40 none)
    for i in ${!HTTP_PORTS_HEX[*]}; do
      if ! cat /proc/net/tcp /proc/net/tcp6 | grep -E ":${HTTP_PORTS_HEX[$i]} [0-9A-F]{8}:[0-9A-F]{4} 0A" >/dev/null 2>&1; then
        break
      fi
    done

    if [ "${HTTP_PORTS[$i]}" == "none" ]; then
      echo "No free http port found in (" "${HTTP_PORTS[@]}" ") - exiting" 2>&1
      exit 1
    fi
    local __resultvar=$1
    local __value="${HTTP_PORTS[$j]}"
    eval $__resultvar="'$__value'"
}

determine_free_https_port() {
    HTTPS_PORTS=(443 8443 4443 none)
    HTTPS_PORTS_HEX=(01BB 208C 114B none)
    for j in ${!HTTPS_PORTS_HEX[*]}; do
      if ! cat /proc/net/tcp /proc/net/tcp6 | grep -E ":${HTTPS_PORTS_HEX[$j]} [0-9A-F]{8}:[0-9A-F]{4} 0A" >/dev/null 2>&1; then
        break
      fi
    done
    if [ "${HTTPS_PORTS[$j]}" == "none" ]; then
      echo "No free http port found in (" "${HTTPS_PORTS[@]}" ") - exiting" 2>&1
      exit 1
    fi
    local __resultvar=$1
    local __value="${HTTPS_PORTS[$j]}"
    eval $__resultvar="'$__value'"
}

start_floxy() {
  if [[ -z "$HTTPS_PORT" ]]; then
      determine_free_https_port "HTTPS_PORT"
  fi
  if [[ -z "$HTTP_PORT" ]]; then
      determine_free_http_port "HTTP_PORT"
  fi

  docker rm -f ${FLOXY_CONTAINER_NAME}
  echo "Binding ${FLOXY_CONTAINER_NAME} to port ${HTTP_PORT}/http and ${HTTPS_PORT}/https"
  docker create \
        --name ${FLOXY_CONTAINER_NAME} \
        --network host \
        --volume flecs-floxy_certs:/etc/nginx/certs \
        --volume flecs-floxy_data:/tmp/floxy \
        --env "FLOXY_HTTP_PORT=${HTTP_PORT}" \
        --env "FLOXY_HTTPS_PORT=${HTTPS_PORT}" \
        --env "FLOXY_FLECS_GATEWAY=$1" \
        "${FLOXY_IMAGE}"
  docker start ${FLOXY_CONTAINER_NAME}
}

# Execute special entrypoint script not in common between flecs and flecs-slim
if [ -f "./special_entrypoint.sh" ]; then
    source ./special_entrypoint.sh;
fi

# verify docker socket is ready
while ! docker version >/dev/null 2>&1; do
    sleep 1
done

create_network "GATEWAY"
start_floxy "${GATEWAY}"

exec flecsd $*
