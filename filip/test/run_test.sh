#!/bin/bash

while [ "$1" != "" ]; do
  case $1 in
    --os)
      shift
      OS="${1}"
      ;;
    --suite)
      shift
      SUITE="${1}"
      ;;
    --all)
      ALL_TESTS=1
      ;;
    *)
      TEST="${1}"
  esac
  shift
done

if [ -z "${OS}" ]; then
  echo "No OS specified"
  exit 1
fi

case ${OS} in
  debian|ubuntu)
    DEBIAN_LIKE=${OS}
    if [ -z "${SUITE}" ]; then
      echo "No SUITE specified"
      exit 1
    fi
    DOCKERFILE=dockerfiles/Dockerfile.debian
    ;;
  *)
    DOCKERFILE=dockerfiles/Dockerfile.${OS}
esac

# if [ -z "${ALL_TESTS}" ] && [ -z "${TEST}" ]; then
#   echo "No TEST specified"
#   exit 1
# fi

DIRNAME=`dirname $(readlink -f ${0})`

docker build -t flecs-installer-test:${OS}-${SUITE} \
  --build-arg OS=${OS} \
  --build-arg SUITE=${SUITE} \
  -f ${DIRNAME}/${DOCKERFILE} ${DIRNAME} || exit 1

docker run --rm \
  -v ${DIRNAME}/..:${DIRNAME}/.. \
  -w ${DIRNAME}/.. \
  flecs-installer-test:${OS}-${SUITE} \
  ./install.sh --yes

docker rmi -f flecs-installer-test:${OS}-${SUITE}
