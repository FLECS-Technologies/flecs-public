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

#include "volume.h"

#include <filesystem>
#include <regex>

#include "util/cxx20/string.h"
#include "util/string/string_utils.h"

namespace FLECS {

volume_t::volume_t() noexcept
    : volume_t{""}
{}

volume_t::volume_t(const std::string& volume_str) noexcept
    : _host{}
    , _container{}
    , _type{volume_type_t::NONE}
{
    const auto parts = split(volume_str, ':');

    if (parts.size() != 2)
    {
        return;
    }

    if (cxx20::starts_with(parts[0], '/'))
    {
        // bind mount
        try
        {
            const auto path = std::filesystem::path{parts[0]};
            if (!path.is_absolute())
            {
                return;
            }
        }
        catch (const std::exception&)
        {
            return;
        }
        _type = BIND_MOUNT;
    }
    else
    {
        // volume
        const auto volume_regex = std::regex{R"(^[a-zA-Z0-9\-_.]+[a-zA-Z0-9]$)"};
        if (!std::regex_match(parts[0], volume_regex))
        {
            return;
        }
        try
        {
            const auto path = std::filesystem::path{parts[1]};
            if (!path.is_absolute())
            {
                return;
            }
        }
        catch (const std::exception&)
        {
            return;
        }
        _type = VOLUME;
    }

    _host = parts[0];
    _container = parts[1];
}

bool volume_t::is_valid() const noexcept
{
    return (!_host.empty() && !_container.empty() && (_type != volume_t::NONE));
}

auto to_json(json_t& json, const volume_t& volume) //
    -> void
{
    json = json_t{{"container", volume._container}, {"host", volume._host}, {"type", stringify(volume._type)}};
}

auto from_json(const json_t& json, volume_t& volume) //
    -> void
{
    json.at("container").get_to(volume._container);
    json.at("host").get_to(volume._host);
    volume._type = volume_type_from_string(json.at("host").get<std::string_view>());
}

} // namespace FLECS
