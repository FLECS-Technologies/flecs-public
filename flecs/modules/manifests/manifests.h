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

#pragma once

#include <memory>
#include <tuple>

#include "flecs/core/flecs.h"
#include "flecs/modules/module_base/module.h"
#include "flecs/util/fs/fs.h"
#include "flecs/util/json/json.h"
#include "flecs/util/yaml/yaml.h"

namespace flecs {
namespace apps {
class key_t;
} // namespace apps

class app_manifest_t;

namespace module {
namespace impl {
class manifests_t;
} // namespace impl

class manifests_t FLECS_FINAL_UNLESS_TESTED : public base_t
{
    friend class factory_t;

public:
    ~manifests_t() override;

    /** @brief Define base_path for local manifest store
     *
     * The base path defines where local manifests will be searched. Given any app_key, a local
     * manifest is expected in the paths: ${base_path}/${app_key.name}/${app_key.version}/manifest.json
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

    /** @brief Migrate base directory to a new directory
     *
     * Moves all manifests from the current ${base_path} to the new path specified as argument.
     * Contents of the new directory will be overwritten on conflict, but otherwise the new
     * directory will not be altered.
     *
     * If migration is successful, the manifest cache will not be cleared and all references
     * retrieved earlier will remain valid. On migration failure, the cache will be cleared
     * and all references are invalidated.
     *
     * @param[in] base_path Path where to move local manifest storage
     *
     * @return true if migration was successful, false otherwise
     */
    auto migrate(const fs::path& base_path) //
        -> bool;

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
    auto contains(const apps::key_t& app_key) const noexcept //
        -> bool;

    /** @brief Obtain a reference to an app manifest in the local manifest cache, if exists
     *
     * Existence of app manifests can be verified by
     *      1) calling the `contains` function for the desired app_key @sa contains
     *      2) checking `operator bool` on the shared_ptr returned by query()
     *
     * Note that query() will trigger loading a local manifest into the cache, if not yet present.
     * If the sole existence in the local manifest cache is of interest, refer to @sa contains()
     * instead.
     *
     * @param[in] app_key to query
     *
     * @return shared_ptr to manifest matching the specified app_key, if exists
     */
    auto query(const apps::key_t& app_key) noexcept //
        -> std::shared_ptr<app_manifest_t>;
    auto query(const apps::key_t& app_key) const noexcept //
        -> std::shared_ptr<const app_manifest_t>;

    using add_result_t = std::tuple<std::shared_ptr<app_manifest_t>, bool>;
    /** @brief Add a manifest to the local manifest store and cache
     */
    auto add(app_manifest_t manifest) //
        -> add_result_t;
    auto add_from_json(const json_t& manifest) //
        -> add_result_t;

    auto add_from_string(std::string_view manifest) //
        -> add_result_t;
    auto add_from_json_string(std::string_view manifest) //
        -> add_result_t;

    auto add_from_file(const fs::path& path) //
        -> add_result_t;
    auto add_from_json_file(const fs::path& path) //
        -> add_result_t;

    auto add_from_console(const apps::key_t& app_key) //
        -> add_result_t;
    auto add_from_url(std::string_view url) //
        -> add_result_t;

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
    auto erase(const apps::key_t& app_key) //
        -> void;
    /** @brief Removes a manifest from the local manifest cache
     *
     * After removing an element, all references obtained since the last call to clear(), the last
     * change of the base_path or the last call to remove() for any app_key are invalidated. To
     * permanently delete a manifest from the local manifest store, refer to @sa erase() instead.
     */
    auto remove(const apps::key_t& app_key) //
        -> void;

    /** @brief Returns the canonical path to a manifest for the specified app_key
     */
    auto path(const apps::key_t& app_key) //
        -> fs::path;

protected:
    manifests_t();

    auto do_init() //
        -> void override;
    auto do_deinit() //
        -> void override;

    std::unique_ptr<impl::manifests_t> _impl;
};

} // namespace module
} // namespace flecs
