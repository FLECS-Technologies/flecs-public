// Copyright 2021-2022 FLECS Technologies GmbH
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#ifndef A1671630_FCCE_4302_B252_4C67A6FF0562
#define A1671630_FCCE_4302_B252_4C67A6FF0562

#include <string>

namespace FLECS {

enum class instance_status_e : char {
    NOT_CREATED = 'n',
    REQUESTED = 'q',
    RESOURCES_READY = 'y',
    CREATED = 'c',
    STOPPED = 's',
    RUNNING = 'r',
    UNKNOWN = 'u',
};

auto to_char(const instance_status_e& instance_status) //
    -> char;

auto to_string(const instance_status_e& instance_status) //
    -> std::string;

auto instance_status_from_string(std::string_view str) //
    -> instance_status_e;

} // namespace FLECS

#endif // A1671630_FCCE_4302_B252_4C67A6FF0562
