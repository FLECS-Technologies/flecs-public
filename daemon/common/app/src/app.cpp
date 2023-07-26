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
    -> std::int64_t
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

auto app_t::installed_size(std::int64_t installed_size) //
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
        {{"_schemaVersion", "2.1.0"},
         {"appKey", static_cast<const app_key_t&>(app)},
         {"status", to_string(app.status())},
         {"desired", to_string(app.desired())},
         {"licenseKey", to_string(app.desired())},
         {"downloadToken", to_string(app.desired())},
         {"installedSize", app.installed_size()}});
}

static auto from_json_v1(const json_t& j, app_t& app) //
    -> void
{
    app = app_t{app_key_t{
        j.at(1).at("app").get<std::string>(), //
        j.at(1).at("version").get<std::string>()}};
    app.status(app_status_from_string(j.at(1).at("status").get<std::string_view>()));
    app.desired(app_status_from_string(j.at(1).at("desired").get<std::string_view>()));
}

static auto from_json_v2(const json_t& j, app_t& app) //
    -> void
{
    app = app_t{j.at("appKey").get<app_key_t>()};
    app.status(app_status_from_string(j.at("status").get<std::string_view>()));
    app.desired(app_status_from_string(j.at("desired").get<std::string_view>()));
    app.installed_size(j.at("installedSize").get<std::int32_t>());
    if (j.at("_schemaVersion").get<std::string_view>() == "2.1.0") {
        app.license_key(j.at("licenseKey").get<std::string>());
        app.download_token(j.at("downloadToken").get<std::string>());
    }
}

auto from_json(const json_t& j, app_t& app) //
    -> void
{
    auto schema_version = std::string_view{"1.0.0"};
    try {
        j.at("_schemaVersion").get_to(schema_version);
    } catch (...) {
    }

    try {
        schema_version[0] == '1' ? from_json_v1(j, app) : from_json_v2(j, app);
    } catch (...) {
        app = app_t{};
    }
}

} // namespace FLECS
