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

#include "flecs/modules/manifests/manifests.h"

namespace flecs {
namespace apps {
class key_t;
} // namespace apps

class app_manifest_t;

namespace module {
namespace impl {

class manifests_t
{
    friend class flecs::module::manifests_t;

public:
    using add_result_t = flecs::module::manifests_t::add_result_t;

    ~manifests_t();

    auto do_base_path(const fs::path& base_path) //
        -> void;
    auto do_base_path() const noexcept //
        -> const fs::path&;

    auto do_migrate(const fs::path& base_path) //
        -> bool;

    auto do_contains(const apps::key_t& app_key) const noexcept //
        -> bool;

    auto do_query_manifest(const apps::key_t& app_key) noexcept //
        -> std::shared_ptr<app_manifest_t>;

    auto do_add_from_url(std::string_view url) //
        -> add_result_t;

    auto do_add_from_string(std::string_view manifest_str) //
        -> add_result_t;

    auto do_add_from_file(const fs::path& path) //
        -> add_result_t;

    auto do_clear() //
        -> void;
    auto do_erase(const apps::key_t& app_key) //
        -> void;
    auto do_remove(const apps::key_t& app_key) //
        -> void;

    auto do_path(const apps::key_t& app_key) //
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
