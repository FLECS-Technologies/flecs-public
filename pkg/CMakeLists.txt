# Copyright 2021-2024 FLECS Technologies GmbH
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

# configure packages

cmake_minimum_required(VERSION 3.25)

set(CPACK_GENERATOR DEB TGZ)

set(CPACK_DEB_COMPONENT_INSTALL 1)
set(CPACK_ARCHIVE_COMPONENT_INSTALL 1)

set(CPACK_PACKAGE_NAME "flecs")
set(CPACK_PACKAGE_VERSION "${FLECS_VERSION_FULL}")

set(CPACK_PACKAGE_ARCHITECTURE "${ARCH}")
set(CPACK_PACKAGE_CONTACT "FLECS Technologies GmbH <info@flecs.tech>")
set(CPACK_PACKAGE_DESCRIPTION_SUMMARY "FLECS is the ultimate open marketplace for industrial automation")
set(CPACK_PACKAGING_INSTALL_PREFIX "/usr/local")

set(CPACK_ARCHIVE_FLECS-TAR-PKG_FILE_NAME ${CPACK_PACKAGE_NAME}_${CPACK_PACKAGE_VERSION}_${CPACK_PACKAGE_ARCHITECTURE})
set(CPACK_ARCHIVE_FILE_EXTENSION "tgz")
set(CPACK_DEBIAN_FLECS-DEB-PKG_PACKAGE_NAME ${CPACK_PACKAGE_NAME})

set(CPACK_DEBIAN_FILE_NAME DEB-DEFAULT)
set(CPACK_DEBIAN_PACKAGE_ARCHITECTURE "${CPACK_PACKAGE_ARCHITECTURE}")
set(CPACK_DEBIAN_PACKAGE_DEPENDS "docker-ce (>= 20.10.5) | docker.io (>= 20.10.5) | docker-compose (>= 1.25.0)")
list(APPEND CPACK_DEBIAN_PACKAGE_CONTROL_EXTRA "${CMAKE_CURRENT_SOURCE_DIR}/DEBIAN/postinst")
list(APPEND CPACK_DEBIAN_PACKAGE_CONTROL_EXTRA "${CMAKE_CURRENT_SOURCE_DIR}/DEBIAN/postrm")
list(APPEND CPACK_DEBIAN_PACKAGE_CONTROL_EXTRA "${CMAKE_CURRENT_SOURCE_DIR}/DEBIAN/prerm")

include(GNUInstallDirs)

# Functions to install common components
function(install_common_files cmp)
    install(CODE
        "execute_process(
            COMMAND sed s/^DOCKER_TAG.*/DOCKER_TAG=${FLECS_VERSION_FULL}/g ${CMAKE_CURRENT_SOURCE_DIR}/fs/bin/flecsd.sh
            OUTPUT_FILE ${CMAKE_CURRENT_BINARY_DIR}/flecsd.sh
        )"
        COMPONENT ${cmp}
        EXCLUDE_FROM_ALL
    )
    install(PROGRAMS ${CMAKE_CURRENT_BINARY_DIR}/flecsd.sh
        DESTINATION ${CMAKE_INSTALL_BINDIR}
        COMPONENT ${cmp}
        EXCLUDE_FROM_ALL
    )
    install(FILES fs/lib/systemd/system/flecs.service
        DESTINATION ${CMAKE_INSTALL_LIBDIR}/systemd/system
        COMPONENT ${cmp}
        EXCLUDE_FROM_ALL
    )
endfunction()

# Install .deb package contents
install_common_files(flecs-deb-pkg)

# Install .tar.gz package contents
install_common_files(flecs-tar-pkg)
install(PROGRAMS fs/etc/init.d/flecs
    DESTINATION ${CMAKE_INSTALL_SYSCONFDIR}/init.d/
    COMPONENT flecs-tar-pkg
    EXCLUDE_FROM_ALL
)
install(FILES fs/etc/flecs/docker-compose.yml
    DESTINATION ${CMAKE_INSTALL_SYSCONFDIR}/flecs/
    COMPONENT flecs-tar-pkg
    EXCLUDE_FROM_ALL
)

set(CPACK_COMPONENTS_ALL "flecs-deb-pkg" "flecs-tar-pkg")

include(CPack)
