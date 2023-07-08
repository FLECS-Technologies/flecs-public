#!/bin/bash

DIRNAME=`dirname $(readlink -f ${0})`

export FLECS_TESTING=1
source ${DIRNAME}/../../install.sh

SED=`which sed`
GREP=""

detect_platform

SED=""
GREP=`which grep`

detect_platform

echo "*** All tests passed"
