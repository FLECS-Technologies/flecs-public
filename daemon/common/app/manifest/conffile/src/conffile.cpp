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

#include "conffile.h"

#include <regex>

#include "util/string/string_utils.h"

namespace FLECS {

conffile_t::conffile_t(const std::string& str)
    : _local{}
    , _container{}
    , _ro{}
    , _init{}
{
    auto parts = split(str, ':');
    if (parts.size() < 2)
    {
        return;
    }

    _local = parts[0];
    _container = parts[1];

    if (parts.size() < 3)
    {
        return;
    }

    auto props = split(parts[2], ',');
    for (const auto& prop : props)
    {
        if (prop == "ro")
        {
            _ro = true;
        }
        else if (prop == "rw")
        {
            // default
        }
        else if (prop == "init")
        {
            _init = true;
        }
        else if (prop == "no_init")
        {
            // default
        }
        else
        {
            std::fprintf(stderr, "Ignoring invalid conffile property '%s'\n", prop.c_str());
        }
    }
}

bool conffile_t::is_valid() const noexcept
{
    // local must be a simple filename, no path
    const auto local_regex = std::regex{"^[^#%&{}\\<>*? $!'\":@+`|=/]+$"};
    // container must be an absolute path
    const auto container_regex = std::regex{"^/[^#%&{}\\<>*? $!'\":@+`|=]+[^/]$"};

    if (!std::regex_match(_local, local_regex))
    {
        return false;
    }

    if (!std::regex_match(_container, container_regex))
    {
        return false;
    }

    return true;
}

void to_json(json_t& j, const conffile_t& conffile)
{
    j = json_t{
        {"local", conffile._local},
        {"container", conffile._container},
        {"ro", conffile._ro},
        {"init", conffile._init},
    };
}

} // namespace FLECS
