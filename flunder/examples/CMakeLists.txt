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

project(flunder.examples)

add_executable(flunder.examples.c
    flunder_client_example.c
)

set_target_properties(flunder.examples.c PROPERTIES LINKER_LANGUAGE C)

set_target_properties(flunder.examples.c PROPERTIES OUTPUT_NAME
    flunder_client_example_c
)

target_link_libraries(flunder.examples.c PRIVATE
    flunder.shared
)

add_executable(flunder.examples.cpp
    flunder_client_example.cpp
)

set_target_properties(flunder.examples.cpp PROPERTIES OUTPUT_NAME
    flunder_client_example_cpp
)

target_link_libraries(flunder.examples.cpp PRIVATE
    flunder.shared
)
