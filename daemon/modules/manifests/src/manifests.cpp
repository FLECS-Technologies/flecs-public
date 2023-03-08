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

#include "manifests.h"

#include "common/app/app_key.h"
#include "common/app/manifest/manifest.h"
#include "factory/factory.h"
#include "impl/manifests_impl.h"

namespace FLECS {

namespace {
register_module_t<module_manifests_t> _reg("manifests");
}

module_manifests_t::module_manifests_t()
    : _impl{new impl::module_manifests_t{this}}
{}

module_manifests_t::~module_manifests_t()
{}

auto module_manifests_t::base_path(const fs::path& base_path) //
    -> void
{
    clear();
    return _impl->do_base_path(std::move(base_path));
}
auto module_manifests_t::base_path() const noexcept //
    -> const fs::path&
{
    return _impl->do_base_path();
}

auto module_manifests_t::migrate(const fs::path& base_path) //
    -> bool
{
    return _impl->do_migrate(base_path);
}

auto module_manifests_t::contains(const app_key_t& app_key) const noexcept //
    -> bool
{
    if (base_path().empty() || !app_key.is_valid()) {
        return false;
    }
    return _impl->do_contains(app_key);
}

auto module_manifests_t::query(const app_key_t& app_key) noexcept //
    -> std::shared_ptr<app_manifest_t>
{
    if (base_path().empty() || !app_key.is_valid()) {
        return {};
    }
    return _impl->do_query_manifest(app_key);
}
auto module_manifests_t::query(const app_key_t& app_key) const noexcept //
    -> std::shared_ptr<const app_manifest_t>
{
    if (base_path().empty() || !app_key.is_valid()) {
        return {};
    }
    return _impl->do_query_manifest(app_key);
}

auto module_manifests_t::add(app_manifest_t manifest) //
    -> std::tuple<std::shared_ptr<app_manifest_t>, bool>
{
    if (base_path().empty() || !manifest.is_valid()) {
        return {};
    }
    return _impl->do_add(std::move(manifest));
}
auto module_manifests_t::add_from_json(const json_t& manifest) //
    -> std::tuple<std::shared_ptr<app_manifest_t>, bool>
{
    return add(app_manifest_t::from_json(manifest));
}
auto module_manifests_t::add_from_yaml(const yaml_t& manifest) //
    -> std::tuple<std::shared_ptr<app_manifest_t>, bool>
{
    return add(app_manifest_t::from_yaml(manifest));
}

auto module_manifests_t::add_from_file(const fs::path& path) //
    -> std::tuple<std::shared_ptr<app_manifest_t>, bool>
{
    auto [manifest, res] = add_from_json_file(path);
    if (res) {
        return {manifest, res};
    }
    return add_from_yaml_file(path);
}
auto module_manifests_t::add_from_json_file(const fs::path& path) //
    -> std::tuple<std::shared_ptr<app_manifest_t>, bool>
{
    return add(app_manifest_t::from_json_file(path));
}
auto module_manifests_t::add_from_yaml_file(const fs::path& path) //
    -> std::tuple<std::shared_ptr<app_manifest_t>, bool>
{
    return add(app_manifest_t::from_yaml_file(path));
}

auto module_manifests_t::add_from_string(std::string_view manifest) //
    -> std::tuple<std::shared_ptr<app_manifest_t>, bool>
{
    auto [manifest_ptr, added] = add_from_json_string(manifest);
    if (manifest_ptr.use_count()) {
        return {manifest_ptr, added};
    }
    return add_from_yaml_string(std::move(manifest));
}
auto module_manifests_t::add_from_json_string(std::string_view manifest) //
    -> std::tuple<std::shared_ptr<app_manifest_t>, bool>
{
    return add_from_json(parse_json(std::move(manifest)));
}
auto module_manifests_t::add_from_yaml_string(std::string_view manifest) //
    -> std::tuple<std::shared_ptr<app_manifest_t>, bool>
{
    return add_from_yaml(yaml_from_string(std::move(manifest)));
}

auto module_manifests_t::add_from_marketplace(const app_key_t& app_key) //
    -> std::tuple<std::shared_ptr<app_manifest_t>, bool>
{
#ifndef NDEBUG
    auto url = std::string{"https://marketplace.flecs.tech:8443/manifests/apps/"};
#else
    auto url = std::string{"https://marketplace.flecs.tech/manifests/apps/"};
#endif // NDEBUG

    url.append(app_key.name());
    url.append("/");
    url.append(app_key.version());
    url.append("/");
    url.append("manifest.yml");

    return add_from_url(url);
}
auto module_manifests_t::add_from_url(std::string_view url) //
    -> std::tuple<std::shared_ptr<app_manifest_t>, bool>
{
    return _impl->do_add_from_url(std::move(url));
}

auto module_manifests_t::clear() //
    -> void
{
    return _impl->do_clear();
}
auto module_manifests_t::erase(const app_key_t& app_key) //
    -> void
{
    if (base_path().empty() || !app_key.is_valid()) {
        return;
    }
    return _impl->do_erase(app_key);
}
auto module_manifests_t::remove(const app_key_t& app_key) //
    -> void
{
    return _impl->do_remove(app_key);
}

auto module_manifests_t::path(const app_key_t& app_key) //
    -> fs::path
{
    if (base_path().empty() || !app_key.is_valid()) {
        return {};
    }
    return _impl->do_path(app_key);
}

auto module_manifests_t::do_init() //
    -> void
{
    return _impl->do_init();
}
auto module_manifests_t::do_deinit() //
    -> void
{
    return _impl->do_deinit();
}

} // namespace FLECS
