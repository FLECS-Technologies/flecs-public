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

#include "flecs/modules/manifests/impl/manifests_impl.h"

#include <cpr/cpr.h>

#include <algorithm>
#include <cstdio>

#include "flecs/common/app/manifest/manifest.h"
#include "flecs/modules/apps/types/app_key.h"
#include "flecs/util/string/literals.h"

namespace flecs {
namespace module {
namespace impl {

manifests_t::manifests_t(flecs::module::manifests_t* parent)
    : _parent{parent}
    , _base_path{}
{}

manifests_t::~manifests_t()
{}

auto manifests_t::do_base_path(const fs::path& base_path) //
    -> void
{
    auto ec = std::error_code{};
    fs::create_directories(base_path, ec);
    if (ec) {
        _base_path.clear();
    }
    _base_path = fs::canonical(base_path, ec);
    if (ec) {
        _base_path.clear();
    }
}

auto manifests_t::do_base_path() const noexcept //
    -> const fs::path&
{
    return _base_path;
}

auto manifests_t::do_migrate(const fs::path& base_path) //
    -> bool
{
    auto to_remove = std::vector<fs::path>{};

    auto ec = std::error_code{};
    auto it = fs::directory_iterator{_base_path, ec};
    for (; it != fs::directory_iterator{}; ++it) {
        if (fs::is_directory(it->path(), ec)) {
            fs::create_directories(base_path / it->path().filename(), ec);
            fs::copy(
                it->path(),
                base_path / it->path().filename(),
                fs::copy_options::recursive | fs::copy_options::overwrite_existing,
                ec);
            if (ec) {
                _parent->clear();
                return false;
            }
            to_remove.push_back(it->path());
        }
    }

    for (const auto& path : to_remove) {
        fs::remove_all(path, ec);
    }
    _parent->base_path(base_path);

    return true;
}

auto manifests_t::do_contains(const apps::key_t& app_key) const noexcept //
    -> bool
{
    return std::find_if(
               _manifests.cbegin(),
               _manifests.cend(),
               [&app_key](const std::shared_ptr<app_manifest_t>& elem) {
                   return elem->app() == app_key.name() && elem->version() == app_key.version();
               }) != _manifests.cend();
}

auto manifests_t::do_query_manifest(const apps::key_t& app_key) noexcept //
    -> std::shared_ptr<app_manifest_t>
{
    auto it = std::find_if(
        _manifests.begin(),
        _manifests.end(),
        [&app_key](const std::shared_ptr<app_manifest_t>& elem) {
            return elem->app() == app_key.name() && elem->version() == app_key.version();
        });
    if (it != _manifests.end()) {
        return *it;
    }

    auto ec = std::error_code{};
    const auto json_path = _base_path / app_key.name() / app_key.version() / "manifest.json";
    if (fs::is_regular_file(json_path, ec)) {
        auto [manifest, res] = _parent->add_from_file(json_path);
        if (res) {
            return manifest;
        }
    }

    return {};
}

auto manifests_t::do_add_from_url(std::string_view url) //
    -> std::tuple<std::shared_ptr<app_manifest_t>, bool>
{
    auto manifest = std::string{};

    auto cbk = [&manifest](const std::string_view& data, intptr_t) {
        if ((manifest.size() + data.size()) > 64_kiB) {
            return false;
        }
        manifest.append(data);
        return true;
    };

    const auto res = cpr::Get(cpr::Url{url.data()}, cpr::WriteCallback{cbk});
    if (res.error.code != cpr::ErrorCode::OK) {
        std::fprintf(
            stderr,
            "Could not download App manifest %s: %d (%s)\n",
            url.data(),
            static_cast<std::underlying_type_t<cpr::ErrorCode>>(res.error.code),
            res.error.message.c_str());
        return {};
    }

    return _parent->add_from_string(manifest);
}

auto manifests_t::do_add_from_string(std::string_view manifest_str) //
    -> std::tuple<std::shared_ptr<app_manifest_t>, bool>
{
    auto manifest = app_manifest_t::from_json_string(manifest_str);
    auto app_key = apps::key_t{manifest.app(), manifest.version()};

    if (_parent->contains(app_key)) {
        auto p = _parent->query(app_key);
        *p = std::move(manifest);
        return {p, false};
    }

    auto p = _manifests.emplace_back(std::make_shared<app_manifest_t>(std::move(manifest)));
    auto ec = std::error_code{};
    fs::create_directories(_parent->path(app_key).parent_path(), ec);
    if (ec) {
        std::fprintf(stderr, "Could not create directory in local manifest store: %d\n", ec.value());
        return {p, false};
    }
    auto file = std::ofstream{_parent->path(app_key)};
    file << manifest_str;
    if (!file) {
        std::fprintf(stderr, "Could not copy manifest to local manifest store\n");
        return {p, false};
    }
    return {p, true};
}

auto manifests_t::do_add_from_file(const fs::path& path) //
    -> std::tuple<std::shared_ptr<app_manifest_t>, bool>
{
    auto manifest = app_manifest_t::from_json_file(path);
    auto app_key = apps::key_t{manifest.app(), manifest.version()};

    if (_parent->contains(app_key)) {
        auto p = _parent->query(app_key);
        *p = std::move(manifest);
        return {p, false};
    }

    auto p = _manifests.emplace_back(std::make_shared<app_manifest_t>(std::move(manifest)));
    auto dest_path = _parent->path(app_key);
    bool copy = true;
    if (fs::exists(dest_path)) {
        try {
            copy = fs::canonical(path) != fs::canonical(dest_path);
        } catch (const fs::filesystem_error& e) {
            std::fprintf(
                stderr,
                "Could not canonize source or destination of manifest file path: %s\n",
                e.what());
            return {p, false};
        }
    }
    if (copy) {
        auto ec = std::error_code{};
        fs::create_directories(dest_path.parent_path(), ec);
        if (ec) {
            std::fprintf(stderr, "Could not create directory in local manifest store: %d\n", ec.value());
            return {p, false};
        }
        fs::copy_file(path, dest_path, ec);
        if (ec) {
            std::fprintf(stderr, "Could not copy manifest file to local manifest store: %d\n", ec.value());
            return {p, false};
        }
    }
    return {p, true};
}

auto manifests_t::do_clear() //
    -> void
{
    _manifests.clear();
}
auto manifests_t::do_erase(const apps::key_t& app_key) //
    -> void
{
    auto ec = std::error_code{};
    fs::remove(_parent->path(app_key), ec);
    if (ec) {
        std::fprintf(
            stderr,
            "Could not delete manifest for %s (%s): %d\n",
            app_key.name().data(),
            app_key.version().data(),
            ec.value());
    }

    _parent->remove(app_key);
}
auto manifests_t::do_remove(const apps::key_t& app_key) //
    -> void
{
    auto it = std::find_if(
        _manifests.begin(),
        _manifests.end(),
        [&app_key](const std::shared_ptr<app_manifest_t>& elem) {
            return elem->app() == app_key.name() && elem->version() == app_key.version();
        });
    if (it != _manifests.end()) {
        _manifests.erase(it);
    }
}

auto manifests_t::do_path(const apps::key_t& app_key) //
    -> fs::path
{
    return _base_path / app_key.name() / app_key.version() / "manifest.json";
}

auto manifests_t::do_init() //
    -> void
{}
auto manifests_t::do_deinit() //
    -> void
{}

} // namespace impl
} // namespace module
} // namespace flecs
