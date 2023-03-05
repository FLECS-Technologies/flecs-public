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

#include <unistd.h>

#include "common/app/app.h"
#include "common/instance/instance.h"
#include "export_manifest.h"
#include "modules/apps/apps.h"
#include "modules/factory/factory.h"
#include "modules/instances/instances.h"
#include "modules/jobs/jobs.h"
#include "modules/version/version.h"
#include "util/archive/archive.h"
#include "util/datetime/datetime.h"
#include "util/sysinfo/sysinfo.h"

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

auto module_flecsport_t::do_exports() const //
    -> std::vector<std::string>
{
    auto res = std::vector<std::string>{};

    auto ec = std::error_code{};
    auto it = fs::directory_iterator("/var/lib/flecs/exports", ec);
    for (; it != fs::directory_iterator{}; ++it) {
        if (!fs::is_regular_file(*it, ec)) {
            continue;
        }
        if ((it->path().extension() == ".gz") && (it->path().stem().extension() == ".tar")) {
            res.push_back(it->path().stem().stem());
        }
    }

    return res;
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
    progress.num_steps(apps.size() + instances.size() + 3);

    auto manifest = export_manifest_t{true};
    manifest.time = time_to_iso(std::atoll(dest_dir.stem().c_str()), precision_e::milliseconds);

    for (auto& app_key : apps) {
        progress.next_step("Exporting App " + to_string(app_key));
        auto [res, message] = _apps_api->export_to(app_key, dest_dir / "apps");
        if (res != 0) {
            auto ec = std::error_code{};
            fs::remove_all(dest_dir, ec);
            return {res, message};
        }
        manifest.contents.apps.push_back(std::move(app_key));
    }

    for (auto& instance_id : instances) {
        progress.next_step("Exporting Instance " + instance_id.hex());
        auto [res, message] = _instances_api->export_to(instance_id, dest_dir / "instances");
        if (res != 0) {
            auto ec = std::error_code{};
            fs::remove_all(dest_dir, ec);
            return {res, message};
        }
        auto instance = _instances_api->query(instance_id);
        manifest.contents.instances.emplace_back(
            instance->id(),
            app_key_t{instance->app_name().data(), instance->app_version().data()});
    }

    progress.next_step("Exporting deployment");
    /** @todo there should be an interface for that */
    auto ec = std::error_code{};
    fs::create_directories(dest_dir / "deployment", ec);
    fs::copy_file(
        "/var/lib/flecs/deployment/docker.json",
        dest_dir / "deployment/docker.json",
        fs::copy_options::overwrite_existing,
        ec);
    if (ec) {
        fs::remove_all(dest_dir, ec);
        return {-1, "Could not export deployment"};
    }

    progress.next_step("Writing manifest");
    auto manifest_file = std::ofstream{dest_dir / "manifest.json"};
    manifest_file << json_t(manifest).dump(4);
    if (!manifest_file) {
        fs::remove_all(dest_dir, ec);
        return {-1, "Could not write manifest"};
    }

    progress.next_step("Creating compressed archive");
    auto archive_name = fs::canonical(dest_dir).string() + ".tar.gz";
    auto res = archive::compress(archive_name, {dest_dir}, dest_dir.parent_path());
    if (res != 0) {
        fs::remove_all(dest_dir, ec);
        return {res, "Could not create compressed archive"};
    }

    fs::remove_all(dest_dir, ec);
    return {0, dest_dir.filename()};
}

auto module_flecsport_t::queue_import_from(fs::path archive) //
    -> job_id_t
{
    auto desc = "Importing " + archive.filename().string();

    auto job = job_t{std::bind(
        &module_flecsport_t::do_import_from,
        this,
        std::move(archive),
        std::placeholders::_1)};

    return _jobs_api->append(std::move(job), std::move(desc));
}
auto module_flecsport_t::do_import_from_sync(fs::path archive) //
    -> result_t
{
    auto _ = job_progress_t{};
    return do_import_from(std::move(archive), _);
}
auto module_flecsport_t::do_import_from(fs::path archive, job_progress_t& progress) //
    -> result_t
{
    progress.num_steps(3);
    progress.next_step("Extracting archive");

    auto basename = archive.filename();
    while (basename.has_extension()) {
        basename = basename.stem();
    }

    /* extract 12345.tar.gz to ${base_path}/12345 */
    auto res = archive::decompress(archive, archive.parent_path() / basename);
    auto ec = std::error_code{};
    fs::remove(archive, ec);
    if (res != 0) {
        return {-1, "Could not extract archive"};
    }

    /* exports contain a directory named like the archive */
    /* ${base_path}/12345/12345 should therefore exist */
    if (!fs::is_directory(archive.parent_path() / basename / basename, ec)) {
        fs::remove_all(archive.parent_path() / basename, ec);
        return {-1, "Archive does not contain a valid export"};
    }

    return {0, {}};
}

} // namespace impl
} // namespace FLECS
