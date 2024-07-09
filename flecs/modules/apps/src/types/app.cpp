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

#include "flecs/modules/apps/types/app.h"

namespace flecs {
namespace apps {

app_t::app_t()
    : key_t{}
    , _installed_size{}
    , _status{status_e::Unknown}
    , _desired{status_e::Unknown}
    , _manifest{}
{}

app_t::app_t(key_t app_key)
    : app_t{std::move(app_key), {}}
{}

app_t::app_t(key_t app_key, std::shared_ptr<app_manifest_t> manifest)
    : key_t{std::move(app_key)}
    , _installed_size{}
    , _status{status_e::Unknown}
    , _desired{status_e::Unknown}
    , _manifest{std::move(manifest)}
{
    if (!is_valid()) {
        *this = app_t{};
    }
}

auto app_t::key() const noexcept //
    -> const key_t&
{
    return static_cast<const key_t&>(*this);
}

auto app_t::installed_size() const noexcept //
    -> std::int64_t
{
    return _installed_size;
}

auto app_t::status() const noexcept //
    -> status_e
{
    return _manifest.expired() ? status_e::Orphaned : _status;
}

auto app_t::desired() const noexcept //
    -> status_e
{
    return _desired;
}

auto app_t::manifest() const noexcept //
    -> std::shared_ptr<app_manifest_t>
{
    return _manifest.lock();
}

auto app_t::installed_size(std::int64_t installed_size) //
    -> void
{
    _installed_size = installed_size;
}

auto app_t::status(status_e status) //
    -> void
{
    _status = status;
}

auto app_t::desired(status_e desired) //
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
    json = json_t({
        {"_schemaVersion", "2.1.0"},
        {"appKey", static_cast<const key_t&>(app)},
        {"status", to_string(app.status())},
        {"desired", to_string(app.desired())},
        {"installedSize", app.installed_size()},
    });
}

static auto from_json_v1(const json_t& j, app_t& app) //
    -> void
{
    app = app_t{key_t{
        j.at(1).at("app").get<std::string>(), //
        j.at(1).at("version").get<std::string>()}};
    app.status(status_from_string(j.at(1).at("status").get<std::string_view>()));
    app.desired(status_from_string(j.at(1).at("desired").get<std::string_view>()));
}

static auto from_json_v2(const json_t& j, app_t& app) //
    -> void
{
    app = app_t{j.at("appKey").get<key_t>()};
    app.status(status_from_string(j.at("status").get<std::string_view>()));
    app.desired(status_from_string(j.at("desired").get<std::string_view>()));
    app.installed_size(j.at("installedSize").get<std::int32_t>());
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

} // namespace apps
} // namespace flecs
