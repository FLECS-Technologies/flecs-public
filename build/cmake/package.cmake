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

set(PACKAGE_DIR ${CMAKE_CURRENT_BINARY_DIR}/pkg)
set(FLECS_INSTALL_DIR /opt/flecs/)

# copy generic pkg directory
install(
    DIRECTORY ${CMAKE_SOURCE_DIR}/build/pkg
    DESTINATION ${CMAKE_CURRENT_BINARY_DIR}
    USE_SOURCE_PERMISSIONS
)

# copy target-specific pkg directory
install(
    DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/pkg
    DESTINATION ${CMAKE_CURRENT_BINARY_DIR}
    USE_SOURCE_PERMISSIONS
    PATTERN "fs" EXCLUDE
)

# copy target-specific additional pkg files, if exist
install(
    DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/pkg/fs/
    DESTINATION ${CMAKE_CURRENT_BINARY_DIR}/pkg/debian
    USE_SOURCE_PERMISSIONS
    OPTIONAL
)

install(
    DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/pkg/fs/
    DESTINATION ${CMAKE_CURRENT_BINARY_DIR}/pkg/opkg
    USE_SOURCE_PERMISSIONS
    OPTIONAL
)

# command for building .deb packages
add_custom_command(
    OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/${PACKAGE}_${VERSION}_${ARCH}.deb
    COMMAND dpkg-deb --root-owner-group -Z gzip --build ${CMAKE_CURRENT_BINARY_DIR}/pkg/debian ${CMAKE_BINARY_DIR}/${PACKAGE}_${VERSION}_${ARCH}.deb
)

# command for building .ipk packages
add_custom_command(
    OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/${PACKAGE}_${VERSION}_${ARCH}.ipk
    COMMAND opkg-build -c -o root -g root ${CMAKE_CURRENT_BINARY_DIR}/pkg/opkg ${CMAKE_BINARY_DIR}
)

# prepare package control fields with target information
# these should not be two separate copy-paste rules, but let's get to that...
add_custom_target(
    ${PACKAGE}_deb-pkg-prepare
    COMMAND sed -i "s/##ARCH##/${ARCH}/g" ${CMAKE_CURRENT_BINARY_DIR}/pkg/debian/DEBIAN/control
    COMMAND sed -i "s/##PACKAGE##/${PACKAGE}/g" ${CMAKE_CURRENT_BINARY_DIR}/pkg/debian/DEBIAN/*
    COMMAND sed -i "s/##VERSION##/${VERSION}/g" ${CMAKE_CURRENT_BINARY_DIR}/pkg/debian/DEBIAN/*
    COMMAND sed -i "s/##DESCRIPTION##/${PACKAGE_DESC}/g" ${CMAKE_CURRENT_BINARY_DIR}/pkg/debian/DEBIAN/*
)

add_custom_target(
    ${PACKAGE}_opkg-pkg-prepare
    COMMAND sed -i "s/##ARCH##/${ARCH}/g" ${CMAKE_CURRENT_BINARY_DIR}/pkg/opkg/DEBIAN/control
    COMMAND sed -i "s/##PACKAGE##/${PACKAGE}/g" ${CMAKE_CURRENT_BINARY_DIR}/pkg/opkg/DEBIAN/*
    COMMAND sed -i "s/##VERSION##/${VERSION}/g" ${CMAKE_CURRENT_BINARY_DIR}/pkg/opkg/DEBIAN/*
    COMMAND sed -i "s/##DESCRIPTION##/${PACKAGE_DESC}/g" ${CMAKE_CURRENT_BINARY_DIR}/pkg/opkg/DEBIAN/*
)

# generic package rule, depends on .deb/.ipk builds
add_custom_target(
    ${PACKAGE}_package
    DEPENDS ${PACKAGE}_deb-pkg-prepare
    DEPENDS ${PACKAGE}_opkg-pkg-prepare
    DEPENDS ${CMAKE_CURRENT_BINARY_DIR}/${PACKAGE}_${VERSION}_${ARCH}.deb
    DEPENDS ${CMAKE_CURRENT_BINARY_DIR}/${PACKAGE}_${VERSION}_${ARCH}.ipk
)

set_property(GLOBAL APPEND PROPERTY PACKAGES ${PACKAGE}_package)
