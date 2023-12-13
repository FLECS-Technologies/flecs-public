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

#include <vector>

#include "daemon/modules/manifests/manifests.h"

namespace flecs {

class app_key_t;
class app_manifest_t;

namespace module {
namespace impl {

class manifests_t
{
    friend class flecs::module::manifests_t;

public:
    ~manifests_t();

    auto do_base_path(const fs::path& base_path) //
        -> void;
    auto do_base_path() const noexcept //
        -> const fs::path&;

    auto do_migrate(const fs::path& base_path) //
        -> bool;

    auto do_contains(const app_key_t& app_key) const noexcept //
        -> bool;

    auto do_query_manifest(const app_key_t& app_key) noexcept //
        -> std::shared_ptr<app_manifest_t>;
    auto do_query_manifest(const app_key_t& app_key) const noexcept //
        -> std::optional<std::reference_wrapper<const app_manifest_t>>;

    auto do_add(app_manifest_t manifest) //
        -> std::tuple<std::shared_ptr<app_manifest_t>, bool>;
    auto do_add_from_url(std::string_view url) //
        -> std::tuple<std::shared_ptr<app_manifest_t>, bool>;

    auto do_clear() //
        -> void;
    auto do_erase(const app_key_t& app_key) //
        -> void;
    auto do_remove(const app_key_t& app_key) //
        -> void;

    auto do_path(const app_key_t& app_key) //
        -> fs::path;

private:
    explicit manifests_t(flecs::module::manifests_t* parent);

    auto do_init() //
        -> void;
    auto do_deinit() //
        -> void;

    flecs::module::manifests_t* _parent;
    fs::path _base_path;
    std::vector<std::shared_ptr<app_manifest_t>> _manifests;
};

} // namespace impl
} // namespace module
} // namespace flecs
