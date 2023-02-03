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
    : app_key_t{}
    , _license_key{}
    , _download_token{}
    , _installed_size{}
    , _status{app_status_e::Unknown}
    , _desired{app_status_e::Unknown}
    , _manifest{}
{}

app_t::app_t(app_key_t app_key)
    : app_t{std::move(app_key), {}}
{}

app_t::app_t(app_key_t app_key, std::shared_ptr<app_manifest_t> manifest)
    : app_key_t{std::move(app_key)}
    , _license_key{}
    , _download_token{}
    , _installed_size{}
    , _status{app_status_e::Unknown}
    , _desired{app_status_e::Unknown}
    , _manifest{std::move(manifest)}
{
    if (!is_valid()) {
        *this = app_t{};
    }
}

auto app_t::key() const noexcept //
    -> const app_key_t&
{
    return static_cast<const app_key_t&>(*this);
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
    return _manifest.expired() ? app_status_e::Orphaned : _status;
}

auto app_t::desired() const noexcept //
    -> app_status_e
{
    return _desired;
}

auto app_t::manifest() const noexcept //
    -> std::shared_ptr<app_manifest_t>
{
    return _manifest.lock();
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

auto app_t::manifest(std::shared_ptr<app_manifest_t> manifest) //
    -> void
{
    _manifest = manifest;
}

auto to_json(json_t& json, const app_t& app) //
    -> void
{
    json = json_t(
        {{"appKey", static_cast<const app_key_t&>(app)},
         {"status", to_string(app._status)},
         {"desired", to_string(app._desired)},
         {"installedSize", app._installed_size}});
}

auto from_json(const json_t& json, app_t& app) //
    -> void
{
    app = app_t{json.at("appKey").get<app_key_t>()};
    app._status = app_status_from_string(json.at("status").get<std::string_view>());
    app._desired = app_status_from_string(json.at("desired").get<std::string_view>());
    json.at("installedSize").get_to(app._installed_size);
}

} // namespace FLECS
