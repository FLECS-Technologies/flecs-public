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

#ifndef A374D934_15CF_4E41_9406_B3368CEA8A94
#define A374D934_15CF_4E41_9406_B3368CEA8A94

#include <string>

namespace FLECS {

enum class app_status_e : char
{
    NOT_INSTALLED = 'n',
    MANIFEST_DOWNLOADED = 'm',
    TOKEN_ACQUIRED = 't',
    IMAGE_DOWNLOADED = 'd',
    INSTALLED = 'i',
    REMOVED = 'r',
    PURGED = 'p',
    UNKNOWN = 'u',
};

inline auto to_char(app_status_e val) //
    -> char
{
    return static_cast<std::underlying_type_t<app_status_e>>(val);
}

auto to_string(app_status_e app_status) //
    -> std::string;

auto app_status_from_string(std::string_view str) //
    -> app_status_e;

} // namespace FLECS

#endif // A374D934_15CF_4E41_9406_B3368CEA8A94
