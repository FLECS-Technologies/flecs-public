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

flecs_add_module(
    MODULE_NAME console
    ADDITIONAL_HEADERS
        impl/console_impl.h
        types/activate_response.h
        types/auth_response.h
        types/base_response.h
        types/error_response.h
        types/feature_flags.h
        types/jwt.h
        types/user.h
        types/validate_response.h
        types.h
    ADDITIONAL_SOURCES
        src/impl/console_impl.cpp
        src/types/activate_response.cpp
        src/types/auth_response.cpp
        src/types/base_response.cpp
        src/types/error_response.cpp
        src/types/feature_flags.cpp
        src/types/jwt.cpp
        src/types/user.cpp
        src/types/validate_response.cpp
)

add_subdirectory(test)
