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

#include "flecs/modules/manifests/manifests.h"

#include "cxxbridge/flecs_core_cxx_bridge/src/lib.rs.h"
#include "cxxbridge/rust/cxx.h"
#include "flecs/common/app/manifest/manifest.h"
#include "flecs/modules/apps/types/app_key.h"
#include "flecs/modules/factory/factory.h"
#include "flecs/modules/manifests/impl/manifests_impl.h"

namespace flecs {
namespace module {

manifests_t::manifests_t()
    : _impl{new impl::manifests_t{this}}
{}

manifests_t::~manifests_t()
{}

auto manifests_t::base_path(const fs::path& base_path) //
    -> void
{
    clear();
    return _impl->do_base_path(std::move(base_path));
}
auto manifests_t::base_path() const noexcept //
    -> const fs::path&
{
    return _impl->do_base_path();
}

auto manifests_t::migrate(const fs::path& base_path) //
    -> bool
{
    return _impl->do_migrate(base_path);
}

auto manifests_t::contains(const apps::key_t& app_key) const noexcept //
    -> bool
{
    if (base_path().empty() || !app_key.is_valid()) {
        return false;
    }
    return _impl->do_contains(app_key);
}

auto manifests_t::query(const apps::key_t& app_key) noexcept //
    -> std::shared_ptr<app_manifest_t>
{
    if (base_path().empty() || !app_key.is_valid()) {
        return {};
    }
    return _impl->do_query_manifest(app_key);
}
auto manifests_t::query(const apps::key_t& app_key) const noexcept //
    -> std::shared_ptr<const app_manifest_t>
{
    if (base_path().empty() || !app_key.is_valid()) {
        return {};
    }
    return _impl->do_query_manifest(app_key);
}

auto manifests_t::add_from_file(const fs::path& path) //
    -> std::tuple<std::shared_ptr<app_manifest_t>, bool>
{
    return _impl->do_add_from_file(path);
}

auto manifests_t::add_from_string(std::string_view manifest) //
    -> std::tuple<std::shared_ptr<app_manifest_t>, bool>
{
    return _impl->do_add_from_string(manifest);
}

auto manifests_t::add_from_console(const apps::key_t& app_key) //
    -> std::tuple<std::shared_ptr<app_manifest_t>, bool>
{
    auto str = std::string{};
    try {
        str = std::string(download_manifest(app_key.name(), app_key.version()));
    } catch (const rust::Error& e) {
        std::cerr << e.what() << "\n";
    }

    return add_from_string(str);
}

auto manifests_t::add_from_url(std::string_view url) //
    -> std::tuple<std::shared_ptr<app_manifest_t>, bool>
{
    return _impl->do_add_from_url(std::move(url));
}

auto manifests_t::clear() //
    -> void
{
    return _impl->do_clear();
}
auto manifests_t::erase(const apps::key_t& app_key) //
    -> void
{
    if (base_path().empty() || !app_key.is_valid()) {
        return;
    }
    return _impl->do_erase(app_key);
}
auto manifests_t::remove(const apps::key_t& app_key) //
    -> void
{
    return _impl->do_remove(app_key);
}

auto manifests_t::path(const apps::key_t& app_key) //
    -> fs::path
{
    if (base_path().empty() || !app_key.is_valid()) {
        return {};
    }
    return _impl->do_path(app_key);
}

auto manifests_t::do_init() //
    -> void
{
    return _impl->do_init();
}
auto manifests_t::do_deinit() //
    -> void
{
    return _impl->do_deinit();
}

} // namespace module
} // namespace flecs
