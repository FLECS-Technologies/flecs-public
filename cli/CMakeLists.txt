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

project(FLECS.cli)

set(SRC_BINARY
    src/main.cpp
)

add_executable(FLECS.cli ${SRC_BINARY})
set_target_properties(FLECS.cli PROPERTIES OUTPUT_NAME flecs)

target_link_libraries(FLECS.cli PRIVATE FLECS.daemon.lib)
