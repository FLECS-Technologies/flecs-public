#!/bin/bash

# Copyright 2021-2022 FLECS Technologies GmbH
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

BASE_URL="https://marketplace.flecs.tech:8443"
AUTH_JSON="/tmp/authenticate.json"
VALIDATE_JSON="/tmp/validate.json"
FLECS_JSON="/tmp/flecs.json"
TICKET_JSON="/tmp/tickets.json"

MP_USER='development-customer@flecs.tech'
MP_PASSWORD='ktYdzgpQDe6YABBz'

authenticate() {
  curl -s -f -X POST \
    -H 'content-type: application/json' \
    -d '{"username":"'${MP_USER}'","password":"'${MP_PASSWORD}'","issueJWT":true}' \
    ${BASE_URL}/api/access/authenticate >${AUTH_JSON}
  if [ $? -ne 0 ]; then
    echo "Could not authenticate with marketplace" 1>&2;
    exit 1;
  fi
}

parse_auth_json() {
  USERNAME=`jq -r '.user.data.user_login' ${AUTH_JSON}`
  if [ "${USERNAME}" == "null" ] || [ "${USERNAME}" == "" ]; then
    echo "Could not determine username from response" 1>&2;
    exit 1;
  fi

  JWT_TOKEN=`jq -r '.jwt.token' ${AUTH_JSON}`
  if [ "${JWT_TOKEN}" == "null" ]  || [ "${JWT_TOKEN}" == "" ]; then
    echo "Could not determine jwt_token from response" 1>&2;
    exit 1;
  fi
}

is_authenticated() {
  AUTHENTICATED="false"
  if [ -f ${AUTH_JSON} ]; then
    # run in subshell to catch 'exit 1', then run in this shell
    # again to populate USERNAME and JWT_TOKEN variables
    (parse_auth_json) && parse_auth_json
    curl -s -f -X POST \
      -H 'content-type: application/json' \
      -d '{"jwt":"'${JWT_TOKEN}'"}' \
      ${BASE_URL}/api/access/jwt/validate >${VALIDATE_JSON}
    if [ $? -eq 0 ]; then
      local IS_VALID=`jq -r '.isValid' ${VALIDATE_JSON}`
      if [ "${IS_VALID}" == "true" ]; then
        AUTHENTICATED="true"
      fi
    fi
  fi
}

get_install_tickets() {
  curl -s -f -X POST \
    -F 'aam-jwt='${JWT_TOKEN}'' \
    ${BASE_URL}/api/license/get-current-user-licenses >${TICKET_JSON}
}

count_tickets() {
  get_install_tickets
  TICKET_COUNT=`jq '[.response.licenses[] | select(.activation_date==null)] | length' ${TICKET_JSON}`
}

get_license_key() {
  get_install_tickets
  LICENSE_KEY=`jq -r '[.response.licenses[] | select(.activation_date==null)][0].license_key' ${TICKET_JSON}`
}

expect_json_value() {
  local VAR=`jq -r "${2}" "${1}"`
  if [ "${VAR}" != "${3}" ]; then
    echo "Error in response, expected" 1>&2
    echo "  ${2} == ${3}, got"
    echo "  ${2} == ${VAR}"
    exit 1
  fi
}

# Check previous authentication and login if invalid/expired
is_authenticated
if [ "${AUTHENTICATED}" != "true" ]; then
  authenticate
  parse_auth_json
fi

echo "Logged in as ${USERNAME} and token ${JWT_TOKEN}"

# TC01: Read app list of freshly installed FLECS
# Expect:
#   - length 2
#   - app "tech.flecs.service-mesh" is "INSTALLED"
#   - single instance of app "tech.flecs.service-mesh" is "RUNNING"
#   - app "tech.flecs.mqtt-bridge" is "INSTALLED"
#   - single instance of app "tech.flecs.mqtt-bridge" is "RUNNING"
curl -s -f -X GET \
  http://localhost/api/app/list >${FLECS_JSON}

echo "TC01: System apps"
echo -n "  Number of installed apps... "
expect_json_value ${FLECS_JSON} '.appList | length' '2' && echo "OK"
echo -n "  Status of invalid app... "
expect_json_value ${FLECS_JSON} '.appList[] | select(.app=="tech.flecs.invalid-app") | .status' '' && echo "OK"

SYSTEM_APPS=(tech.flecs.mqtt-bridge tech.flecs.service-mesh)
for APP in ${SYSTEM_APPS[*]}; do
  echo -n "  Status of ${APP}... "
  expect_json_value ${FLECS_JSON} '.appList[] | select(.app=="'${APP}'") | .status' 'installed' && echo "OK"
  echo -n "  Number of instances of ${APP}... "
  expect_json_value ${FLECS_JSON} '.appList[] | select(.app=="'${APP}'") | .instances | length' '1' && echo "OK"
  echo -n "  Status of ${APP} instance... "
  expect_json_value ${FLECS_JSON} '.appList[] | select(.app=="'${APP}'") | .instances[0] | .status' 'running' && echo "OK"
done

# TC02: Pass login data to FLECS daemon
# Expect:
#   - "OK" response
echo "TC02: Login"
curl -s -f -X POST \
  -H 'content-type: application/json' \
  -d '{"token":"'${JWT_TOKEN}'","user":"'${USERNAME}'"}' \
  http://localhost/api/marketplace/login >${FLECS_JSON}

echo -n "  Send login service to FLECS daemon... "
expect_json_value ${FLECS_JSON} ".additionalInfo" "OK" && echo "OK"

# TC03: App installation
# USER_APPS=(org.mosquitto.broker org.openjsf.node-red io.anyviz.cloudadapter)
# USER_APP_VERSIONS=(2.0.14-openssl 2.2.2 0.9.3.3)
USER_APPS=(org.mosquitto.broker)
USER_APPS_VERSIONS=(2.0.14-openssl)

echo "TC03: App installation"
for i in "${!USER_APPS[@]}"; do
  count_tickets
  echo "${TICKET_COUNT} tickets available"

  get_license_key
  echo "Installing ${USER_APPS[$i]} ${USER_APPS_VERSIONS[$i]} with key ${LICENSE_KEY}"

  curl -s -f -X POST \
    -H 'content-type: application/json' \
    -d '{"app":"'${USER_APPS[$i]}'","version":"'${USER_APPS_VERSIONS[$i]}'","licenseKey":"'${LICENSE_KEY}'"}' \
    http://localhost/api/app/install >${FLECS_JSON}

  TICKET_COUNT_OLD=${TICKET_COUNT}
  count_tickets

  curl -s -f -X GET \
    http://localhost/api/app/list >${FLECS_JSON}
  echo -n "  Status of ${USER_APPS[$i]}... "
  expect_json_value ${FLECS_JSON} '.appList[] | select(.app=="'${USER_APPS[$i]}'") | .status' 'installed' && echo "OK"
done
