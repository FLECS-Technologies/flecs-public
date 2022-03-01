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

#include "app/env_var/env_var.h"

#include <regex>

#include "util/string/string_utils.h"

namespace FLECS {

bool env_var_t::is_valid() const noexcept
{
    const auto name_regex = std::regex{"[a-zA-Z]+[a-zA-Z0-9_]*"};

    if (std::regex_match(_var, name_regex))
    {
        return true;
    }

    return false;
}

mapped_env_var_t::mapped_env_var_t(const std::string& str)
    : _env_var{}
    , _value{}
{
    const auto parts = split(str, ':');
    if (parts.size() < 2)
    {
        return;
    }

    _env_var = parts[0];
    _value = parts[1];
}

} // namespace FLECS
