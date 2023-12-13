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

project(daemon.lib)

add_library(daemon.lib SHARED
    src/libflecs.cpp
    src/impl/libflecs_impl.cpp
    libflecs.h
    impl/libflecs_impl.h
)

set_target_properties(daemon.lib PROPERTIES OUTPUT_NAME FLECS.daemon)

target_link_libraries(daemon.lib PRIVATE
    daemon.modules.module_base
    cpr::cpr
)

add_subdirectory(test)
