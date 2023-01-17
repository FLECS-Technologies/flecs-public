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

#pragma once

#include <functional>
#include <memory>
#include <optional>
#include <tuple>

#include "core/flecs.h"
#include "module_base/module.h"
#include "util/fs/fs.h"
#include "util/json/json.h"
#include "util/yaml/yaml.h"

namespace FLECS {

class app_key_t;
class app_manifest_t;

namespace impl {
class module_manifests_t;
} // namespace impl

class module_manifests_t FLECS_FINAL_UNLESS_TESTED : public module_t
{
    friend class module_factory_t;

public:
    ~module_manifests_t() override;

    /** @brief Define base_path for local manifest store
     *
     * The base path defines where local manifests will be searched. Given any app_key, a local
     * manifest is expected in these paths:
     *      1) ${base_path}/${app_key.name}/${app_key.version}/manifest.json
     *      2) ${base_path}/${app_key.name}/${app_key.version}/manifest.yml
     * If both .json and .yml exist, json will be preferred.
     *
     * Upon changing the base path, the manifest cache will be cleared, invalidating all references
     * retrieved since the base_path has last been changed, the last call to remove() for any
     * app_key or the local manifest cache has last been cleared. @sa remove() @sa clear()
     *
     * @param[in] base_path Path in which to search for App manifests
     *
     * @return none
     */
    auto base_path(const fs::path& base_path) //
        -> void;
    auto base_path() const noexcept //
        -> const fs::path&;

    /** @brief Verify the existence of an app_key in the local manifest cache
     *
     * If an app_key exists in the local manifest store, but has not yet been loaded into the local
     * manifest cache, contains() will return false. To load an App manifest into the cache, refer
     * to @sa query()
     *
     * @param[in] app_key key to search for
     *
     * @return true if app_key is present, false otherwise
     */
    auto contains(const app_key_t& app_key) const noexcept //
        -> bool;

    /** @brief Obtain a reference to an app manifest in the local manifest cache, if exists
     *
     * Existence of app manifests can be verified by
     *      1) calling the `contains` function for the desired app_key @sa contains
     *      2) calling `has_value()` on the optional returned by query()
     *
     * Note that query() will trigger loading a local manifest into the cache, if not yet present.
     * If the sole existence in the local manifest cache is of interest, refer to @sa contains()
     * instead.
     *
     * @param[in] app_key to query
     *
     * @return optional reference to manifest matching the specified app_key
     */
    auto query(const app_key_t& app_key) noexcept //
        -> std::optional<std::reference_wrapper<app_manifest_t>>;
    auto query(const app_key_t& app_key) const noexcept //
        -> std::optional<std::reference_wrapper<const app_manifest_t>>;

    /** @brief Add a manifest to the local manifest store and cache
     */
    auto add(app_manifest_t manifest) //
        -> std::tuple<std::optional<std::reference_wrapper<app_manifest_t>>, bool>;
    auto add_from_json(const json_t& manifest) //
        -> std::tuple<std::optional<std::reference_wrapper<app_manifest_t>>, bool>;
    auto add_from_yaml(const yaml_t& manifest) //
        -> std::tuple<std::optional<std::reference_wrapper<app_manifest_t>>, bool>;

    auto add_from_string(std::string_view manifest) //
        -> std::tuple<std::optional<std::reference_wrapper<app_manifest_t>>, bool>;
    auto add_from_json_string(std::string_view manifest) //
        -> std::tuple<std::optional<std::reference_wrapper<app_manifest_t>>, bool>;
    auto add_from_yaml_string(std::string_view manifest) //
        -> std::tuple<std::optional<std::reference_wrapper<app_manifest_t>>, bool>;

    auto add_from_file(const fs::path& path) //
        -> std::tuple<std::optional<std::reference_wrapper<app_manifest_t>>, bool>;
    auto add_from_json_file(const fs::path& path) //
        -> std::tuple<std::optional<std::reference_wrapper<app_manifest_t>>, bool>;
    auto add_from_yaml_file(const fs::path& path) //
        -> std::tuple<std::optional<std::reference_wrapper<app_manifest_t>>, bool>;

    auto add_from_marketplace(const app_key_t& app_key) //
        -> std::tuple<std::optional<std::reference_wrapper<app_manifest_t>>, bool>;
    auto add_from_url(std::string_view url) //
        -> std::tuple<std::optional<std::reference_wrapper<app_manifest_t>>, bool>;

    /** @brief Clears the local manifest cache
     *
     * After clearing the cache, all references obtained since the last call to clear(), the last
     * change of the base_path or the last call to remove() for any app_key are invalidated.
     */
    auto clear() //
        -> void;
    /** @brief Erases a manifest from the local manifest cache and store
     *
     * After erasing an element, all references obtained since the last call to clear(), the last
     * change of the base_path or the last call to remove() for any app_key are invalidated. The
     * manifest is also permanently removed from the local manifest store. To only remove an element
     * from cache, refer to @sa remove() instead.
     *
     */
    auto erase(const app_key_t& app_key) //
        -> void;
    /** @brief Removes a manifest from the local manifest cache
     *
     * After removing an element, all references obtained since the last call to clear(), the last
     * change of the base_path or the last call to remove() for any app_key are invalidated. To
     * permanently delete a manifest from the local manifest store, refer to @sa erase() instead.
     */
    auto remove(const app_key_t& app_key) //
        -> void;

    /** @brief Returns the canonical path to a manifest for the specified app_key
     */
    auto path(const app_key_t& app_key) //
        -> fs::path;

protected:
    module_manifests_t();

    auto do_init() //
        -> void override;
    auto do_deinit() //
        -> void override;

    std::unique_ptr<impl::module_manifests_t> _impl;
};

} // namespace FLECS
