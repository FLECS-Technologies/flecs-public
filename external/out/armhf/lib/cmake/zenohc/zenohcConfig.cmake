#
# Copyright (c) 2022 ZettaScale Technology.
#
# This program and the accompanying materials are made available under the
# terms of the Eclipse Public License 2.0 which is available at
# http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
# which is available at https://www.apache.org/licenses/LICENSE-2.0.
#
# SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
#
# Contributors:
#   ZettaScale Zenoh team, <zenoh@zettascale.tech>
#


####### Expanded from @PACKAGE_INIT@ by configure_package_config_file() #######
####### Any changes to this file will be overwritten by the next CMake run ####
####### The input file was PackageConfig.cmake.in                            ########

get_filename_component(PACKAGE_PREFIX_DIR "${CMAKE_CURRENT_LIST_DIR}/../../../" ABSOLUTE)

macro(set_and_check _var _file)
  set(${_var} "${_file}")
  if(NOT EXISTS "${_file}")
    message(FATAL_ERROR "File or directory ${_file} referenced by variable ${_var} does not exist !")
  endif()
endmacro()

macro(check_required_components _NAME)
  foreach(comp ${${_NAME}_FIND_COMPONENTS})
    if(NOT ${_NAME}_${comp}_FOUND)
      if(${_NAME}_FIND_REQUIRED_${comp})
        set(${_NAME}_FOUND FALSE)
      endif()
    endif()
  endforeach()
endmacro()

####################################################################################

add_library(__zenohc_static STATIC IMPORTED GLOBAL)
add_library(zenohc::static ALIAS __zenohc_static)
set_property(TARGET __zenohc_static PROPERTY IMPORTED_LOCATION "/home/alex/git/flecs-public/external/out/armhf/lib/libzenohc.a")

add_library(__zenohc_lib STATIC IMPORTED GLOBAL)
add_library(zenohc::lib ALIAS __zenohc_lib)
set_property(TARGET __zenohc_lib PROPERTY IMPORTED_LOCATION "/home/alex/git/flecs-public/external/out/armhf/lib/libzenohc.so")
target_include_directories(__zenohc_lib INTERFACE "/home/alex/git/flecs-public/external/out/armhf/include/zenohc")
