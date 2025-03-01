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

#include "flecs/modules/flecsport/flecsport.h"

namespace flecs {
namespace jobs {
class progress_t;
} // namespace jobs

namespace module {

class apps_t;
class jobs_t;
class instances_t;

namespace impl {

class flecsport_t
{
    friend class flecs::module::flecsport_t;

private:
    explicit flecsport_t(flecs::module::flecsport_t* parent);

    auto do_module_load(const fs::path&) -> //
        result_t;

    auto do_init() //
        -> void;

    auto do_exports() const //
        -> std::vector<std::string>;

    auto queue_export_to(
        std::vector<apps::key_t> apps, std::vector<instances::id_t> instances, fs::path dest_dir) //
        -> jobs::id_t;
    auto do_export_to_sync(
        std::vector<apps::key_t> apps, std::vector<instances::id_t> instances, fs::path dest_dir) //
        -> result_t;
    auto do_export_to(
        std::vector<apps::key_t> apps,
        std::vector<instances::id_t> instances,
        fs::path dest_dir,
        jobs::progress_t& progress) //
        -> result_t;

    auto queue_import_from(fs::path archive) //
        -> jobs::id_t;
    auto do_import_from_sync(fs::path archive) //
        -> result_t;
    auto do_import_from(fs::path archive, jobs::progress_t& progress) //
        -> result_t;

    flecs::module::flecsport_t* _parent;

    std::shared_ptr<flecs::module::apps_t> _apps_api;
    std::shared_ptr<flecs::module::instances_t> _instances_api;
    std::shared_ptr<flecs::module::jobs_t> _jobs_api;
};

} // namespace impl
} // namespace module
} // namespace flecs
