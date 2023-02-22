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

#include "impl/flecsport_impl.h"

#include "common/app/app.h"
#include "common/instance/instance_id.h"
#include "modules/apps/apps.h"
#include "modules/factory/factory.h"
#include "modules/instances/instances.h"
#include "modules/jobs/jobs.h"
#include "util/archive/archive.h"

namespace FLECS {
namespace impl {

module_flecsport_t::module_flecsport_t(FLECS::module_flecsport_t* parent)
    : _parent{parent}
    , _apps_api{}
    , _instances_api{}
    , _jobs_api{}
{}

auto module_flecsport_t::do_init() //
    -> void
{
    _apps_api = std::dynamic_pointer_cast<FLECS::module_apps_t>(api::query_module("apps"));
    _instances_api =
        std::dynamic_pointer_cast<FLECS::module_instances_t>(api::query_module("instances"));
    _jobs_api = std::dynamic_pointer_cast<FLECS::module_jobs_t>(api::query_module("jobs"));
}

auto module_flecsport_t::queue_export_to(
    std::vector<app_key_t> apps, std::vector<instance_id_t> instances, fs::path dest_dir) //
    -> job_id_t
{
    auto job = job_t{std::bind(
        &module_flecsport_t::do_export_to,
        this,
        std::move(apps),
        std::move(instances),
        std::move(dest_dir),
        std::placeholders::_1)};

    return _jobs_api->append(std::move(job), "Creating export");
}
auto module_flecsport_t::do_export_to_sync(
    std::vector<app_key_t> apps,
    std::vector<instance_id_t> instances,
    fs::path dest_dir) //
    -> result_t
{
    auto _ = job_progress_t{};
    return do_export_to(std::move(apps), std::move(instances), std::move(dest_dir), _);
}
auto module_flecsport_t::do_export_to(
    std::vector<app_key_t> apps,
    std::vector<instance_id_t> instances,
    fs::path dest_dir,
    job_progress_t& progress) //
    -> result_t
{
    progress.num_steps(apps.size() + instances.size() + 1);
    for (auto& app_key : apps) {
        progress.next_step("Exporting App " + to_string(app_key));
        auto [res, message] = _apps_api->export_to(app_key, dest_dir / "apps");
        if (res != 0) {
            auto ec = std::error_code{};
            fs::remove_all(dest_dir, ec);
            return {res, message};
        }
    }
    for (auto& instance_id : instances) {
        progress.next_step("Exporting Instance " + instance_id.hex());
        auto [res, message] = _instances_api->export_to(instance_id, dest_dir / "instances");
        if (res != 0) {
            auto ec = std::error_code{};
            fs::remove_all(dest_dir, ec);
            return {res, message};
        }
    }

    progress.next_step("Creating compressed archive");
    auto archive_name = fs::canonical(dest_dir).string() + ".tar.gz";
    auto res = archive::compress(archive_name, {dest_dir}, dest_dir.parent_path());
    if (res != 0) {
        auto ec = std::error_code{};
        fs::remove_all(dest_dir, ec);
        return {res, "Could not create compressed archive"};
    }

    return {0, {}};
}

} // namespace impl
} // namespace FLECS
