// Copyright 2021-2023 FLECS Technologies GmbH
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

namespace flecs {

conffile_t::conffile_t(std::string_view str)
    : _local{}
    , _container{}
    , _ro{}
    , _init{}
{
    auto parts = split(str, ':');
    if (parts.size() < 2) {
        return;
    }

    _local = parts[0];
    _container = parts[1];

    if (parts.size() < 3) {
        return;
    }

    auto props = split(parts[2], ',');
    for (const auto& prop : props) {
        if (prop == "ro") {
            _ro = true;
        } else if (prop == "rw") {
            // default
        } else if (prop == "init") {
            _init = true;
        } else if (prop == "no_init") {
            // default
        } else {
            std::fprintf(stderr, "Ignoring invalid conffile property '%s'\n", prop.c_str());
        }
    }
}

auto conffile_t::local() const noexcept //
    -> const std::string&
{
    return _local;
}
auto conffile_t::local(std::string local) //
    -> void
{
    _local = local;
}

auto conffile_t::container() const noexcept //
    -> const std::string&
{
    return _container;
}
auto conffile_t::container(std::string container) //
    -> void
{
    _container = container;
}

auto conffile_t::ro() const noexcept //
    -> bool
{
    return _ro;
}
auto conffile_t::ro(bool ro) //
    -> void
{
    _ro = ro;
}

auto conffile_t::init() const noexcept //
    -> bool
{
    return _init;
}
auto conffile_t::init(bool init) //
    -> void
{
    _init = init;
}

bool conffile_t::is_valid() const noexcept
{
    // local must be a simple filename, no path
    const auto local_regex = std::regex{"^[^#%&{}\\<>*? $!'\":@+`|=/]+$"};
    // container must be an absolute path
    const auto container_regex = std::regex{"^/[^#%&{}\\<>*? $!'\":@+`|=]+[^/]$"};

    if (!std::regex_match(_local, local_regex)) {
        return false;
    }

    if (!std::regex_match(_container, container_regex)) {
        return false;
    }

    return true;
}

auto to_json(json_t& j, const conffile_t& conffile) //
    -> void
{
    j = json_t(to_string(conffile));
}

auto from_json(const json_t& j, conffile_t& conffile) //
    -> void
{
    try {
        conffile = conffile_t{j.get<std::string_view>()};
    } catch (...) {
        conffile = conffile_t{};
    }
}

auto to_string(const conffile_t& conffile) //
    -> std::string
{
    return stringify_delim(
        ':',
        conffile.local(),
        conffile.container(),
        stringify_delim(',', conffile.ro() ? "ro" : "rw", conffile.init() ? "init" : "no_init"));
}

auto operator<(const conffile_t& lhs, const conffile_t& rhs) //
    -> bool
{
    return lhs.local() < rhs.local();
}

auto operator<=(const conffile_t& lhs, const conffile_t& rhs) //
    -> bool
{
    return !(lhs > rhs);
}

auto operator>(const conffile_t& lhs, const conffile_t& rhs) //
    -> bool
{
    return rhs < lhs;
}

auto operator>=(const conffile_t& lhs, const conffile_t& rhs) //
    -> bool
{
    return !(lhs < rhs);
}

auto operator==(const conffile_t& lhs, const conffile_t& rhs) //
    -> bool
{
    return lhs.local() == rhs.local();
}

auto operator!=(const conffile_t& lhs, const conffile_t& rhs) //
    -> bool
{
    return !(lhs == rhs);
}

} // namespace flecs
