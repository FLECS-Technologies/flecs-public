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

#include "app.h"

namespace FLECS {

app_t::app_t()
    : app_manifest_t{}
    , _license_key{}
    , _download_token{}
    , _installed_size{}
    , _status{}
    , _desired{}
{}

app_t::app_t(const fs::path& manifest_path, app_status_e status, app_status_e desired)
    : app_manifest_t{app_manifest_t::from_yaml_file(manifest_path)}
    , _license_key{}
    , _download_token{}
    , _installed_size{}
    , _status{status}
    , _desired{desired}
{
    if (!is_valid()) {
        *this = app_t{};
    }

    _key = app_key_t{app(), version()};
}

app_t::app_t(const std::string& manifest_string, app_status_e status, app_status_e desired)
    : app_manifest_t{app_manifest_t::from_yaml_string(manifest_string)}
    , _status{status}
    , _desired(desired)
{
    if (!is_valid()) {
        *this = app_t{};
    }

    _key = app_key_t{app(), version()};
}

auto app_t::key() const noexcept //
    -> const app_key_t&
{
    return _key;
}

auto app_t::download_token() const noexcept //
    -> const std::string&
{
    return _download_token;
}

auto app_t::installed_size() const noexcept //
    -> std::int32_t
{
    return _installed_size;
}

auto app_t::license_key() const noexcept //
    -> const std::string&
{
    return _license_key;
}

auto app_t::status() const noexcept //
    -> app_status_e
{
    return _status;
}

auto app_t::desired() const noexcept //
    -> app_status_e
{
    return _desired;
}

auto app_t::download_token(std::string download_token) //
    -> void
{
    _download_token = std::move(download_token);
}

auto app_t::installed_size(std::int32_t installed_size) //
    -> void
{
    _installed_size = installed_size;
}

auto app_t::license_key(std::string license_key) //
    -> void
{
    _license_key = license_key;
}

auto app_t::status(app_status_e status) //
    -> void
{
    _status = status;
}

auto app_t::desired(app_status_e desired) //
    -> void
{
    _desired = desired;
}

auto to_json(json_t& json, const app_t& app) //
    -> void
{
    json = json_t(
        {{"app_key", app._key},
         {"status", to_string(app._status)},
         {"desired", to_string(app._desired)},
         {"installedSize", app._installed_size}});
}

auto from_json(const json_t& json, app_t& app) //
    -> void
{
    json.at("app_key").get_to(app._key);
    app._status = app_status_from_string(json.at("status").get<std::string_view>());
    app._desired = app_status_from_string(json.at("desired").get<std::string_view>());
    json.at("installedSize").get_to(app._installed_size);
}

} // namespace FLECS
