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

#include "flecs/modules/apps/impl/apps_impl.h"

#include <cpr/cpr.h>

#include <algorithm>
#include <fstream>
#include <sstream>

#include "cxxbridge/flecs_core_cxx_bridge/src/lib.rs.h"
#include "cxxbridge/rust/cxx.h"
#include "flecs/api/api.h"
#include "flecs/common/app/manifest/manifest.h"

#ifdef FLECS_MOCK_MODULES
#include "flecs/modules/instances/__mocks__/instances.h"
#include "flecs/modules/jobs/__mocks__/jobs.h"
#include "flecs/modules/manifests/__mocks__/manifests.h"
#else // FLECS_MOCK_MODULES
#include "flecs/modules/instances/instances.h"
#include "flecs/modules/jobs/jobs.h"
#include "flecs/modules/manifests/manifests.h"
#endif // FLECS_MOCK_MODULES
#include "flecs/modules/deployments/deployments.h"
#include "flecs/modules/factory/factory.h"
#include "flecs/modules/instances/types/instance.h"
#include "flecs/util/cxx23/string.h"
#include "flecs/util/fs/fs.h"
#include "flecs/util/json/json.h"
#include "flecs/util/process/process.h"
#include "flecs/util/string/string_utils.h"

using std::operator""s;

namespace flecs {
namespace module {
namespace impl {

apps_t::apps_t(flecs::module::apps_t* parent)
    : _parent{parent}
    , _apps{}
    , _apps_mutex{}
    , _instances_api{}
    , _manifests_api{}
    , _jobs_api{}
{}

apps_t::~apps_t()
{}

auto apps_t::do_module_init() //
    -> void
{
    auto ec = std::error_code{};
    if (fs::is_directory("/var/lib/flecs/apps", ec)) {
        _manifests_api->base_path("/var/lib/flecs/apps");
        _manifests_api->migrate("/var/lib/flecs/manifests/");
    } else {
        _manifests_api->base_path("/var/lib/flecs/manifests/");
    }

    for (auto& app : _apps) {
        auto manifest = _manifests_api->query(app->key());
        if (manifest) {
            app->manifest(std::move(manifest));
        }
    }

    for (auto id : _instances_api->instance_ids()) {
        auto instance = _instances_api->query(id);
        instance->app(
            _parent->query(apps::key_t{instance->app_name().data(), instance->app_version().data()}));
    }
}

auto apps_t::do_load(const fs::path& base_path) //
    -> result_t
{
    _instances_api = std::dynamic_pointer_cast<flecs::module::instances_t>(api::query_module("instances"));
    _manifests_api = std::dynamic_pointer_cast<flecs::module::manifests_t>(api::query_module("manifests"));
    _jobs_api = std::dynamic_pointer_cast<flecs::module::jobs_t>(api::query_module("jobs"));

    const auto json_path = base_path / "apps.json";
    auto json_file = std::ifstream{json_path};

    if (!json_file.good()) {
        return {-1, "Could not open apps.json for reading"};
    }

    auto apps_json = parse_json(json_file);
    try {
        _apps.reserve(apps_json.size());
        for (const auto& app : apps_json) {
            _apps.push_back(std::make_shared<apps::app_t>(app.get<apps::app_t>()));
        }
    } catch (const std::exception& ex) {
        _apps.clear();
        return {-1, "Could not read contents of apps.json"};
    }

    return {0, {}};
}

auto apps_t::do_module_start() //
    -> void
{}

auto apps_t::do_save(const fs::path& base_path) const //
    -> result_t
{
    auto ec = std::error_code{};
    fs::create_directories(base_path, ec);
    if (ec) {
        return {-1, "Could not create directory"};
    }

    const auto json_path = base_path / "apps.json";
    auto json_file = std::ofstream{json_path, std::ios_base::out | std::ios_base::trunc};
    if (!json_file.good()) {
        return {-1, "Could not open apps.json for writing"};
    }

    auto apps_json = json_t::array();
    for (const auto& app : _apps) {
        apps_json.push_back(*app);
    }

    json_file << apps_json;
    if (!json_file.good()) {
        return {-1, "Could not write apps.json"};
    }

    return {0, {}};
}

auto apps_t::do_app_keys(const apps::key_t& app_key) const //
    -> std::vector<apps::key_t>
{
    auto res = std::vector<apps::key_t>{};

    for (const auto& app : _apps) {
        const auto apps_match = app_key.name().empty() || (app_key.name() == app->key().name());
        const auto versions_match = app_key.name().empty() || app_key.version().empty() ||
                                    (app_key.version() == app->key().version());
        if (apps_match && versions_match) {
            res.push_back(app->key());
        }
    }

    return res;
}

auto apps_t::queue_install_from_marketplace(apps::key_t app_key) //
    -> jobs::id_t
{
    auto desc = "Installation of "s + to_string(app_key);

    auto job = jobs::job_t{
        std::bind(&apps_t::do_install_from_marketplace, this, std::move(app_key), std::placeholders::_1)};

    return _jobs_api->append(std::move(job), std::move(desc));
}

auto apps_t::queue_install_many_from_marketplace(std::vector<apps::key_t> app_keys) //
    -> jobs::id_t
{
    auto desc = "Installation of "s + to_string(app_keys.size()) + " apps";

    auto job = jobs::job_t{std::bind(
        &apps_t::do_install_many_from_marketplace,
        this,
        std::move(app_keys),
        std::placeholders::_1)};

    return _jobs_api->append(std::move(job), std::move(desc));
}

auto apps_t::do_install_from_marketplace_sync(apps::key_t app_key) //
    -> result_t
{
    auto _ = jobs::progress_t{};
    return do_install_from_marketplace(std::move(app_key), _);
}

auto apps_t::do_install_many_from_marketplace_sync(std::vector<apps::key_t> app_keys) //
    -> result_t
{
    auto _ = jobs::progress_t{};
    return do_install_many_from_marketplace(std::move(app_keys), _);
}

auto apps_t::do_install_from_marketplace(apps::key_t app_key, jobs::progress_t& progress) //
    -> result_t
{
    progress.num_steps(6);
    return install_from_marketplace(std::move(app_key), progress);
}

auto apps_t::do_install_many_from_marketplace(
    std::vector<apps::key_t> app_keys, jobs::progress_t& progress) //
    -> result_t
{
    static constexpr std::int16_t TOTAL_STEPS_PER_APP = 9;
    progress.num_steps(TOTAL_STEPS_PER_APP * app_keys.size());

    auto failed_apps = std::vector<std::pair<apps::key_t, std::string>>{};
    for (size_t i = 0; i < app_keys.size(); i++) {
        auto [app_result, message] = install_from_marketplace(app_keys[i], progress);
        if (app_result == 0) {
            progress.next_step(
                "Creating instance of " + app_keys[i].name() + " (" + app_keys[i].version() + ")");
            std::tie(app_result, message) = _instances_api->create(app_keys[i].name(), app_keys[i].version());
        }
        if (app_result == 0) {
            progress.next_step(
                "Starting instance " + message + "of " + app_keys[i].name() + " (" + app_keys[i].version() +
                ")");
            std::tie(app_result, message) = _instances_api->start(instances::id_t{message});
        }
        if (app_result != 0) {
            progress.skip_to_step(TOTAL_STEPS_PER_APP * (i + 1));
            failed_apps.emplace_back(app_keys[i], message);
        }
    }
    if (!failed_apps.empty()) {
        auto message = std::stringstream{};
        message << "Failed to install the following " << failed_apps.size() << " app installations out of "
                << app_keys.size() << ": ";
        std::for_each(failed_apps.begin(), failed_apps.end() - 1, [&message](auto& app_key) {
            message << to_string(std::get<0>(app_key)) << " [" << std::get<1>(app_key) << "], ";
        });
        message << to_string(std::get<0>(failed_apps.back())) << " [" << std::get<1>(failed_apps.back())
                << "]";
        return {-1, message.str()};
    }
    return {0, {}};
}

auto apps_t::install_from_marketplace(apps::key_t app_key, jobs::progress_t& progress) //
    -> result_t
{
    progress.next_step("Downloading manifest");
    // Download App manifest and forward to manifest installation, if download successful
    const auto [manifest, _] = _manifests_api->add_from_console(app_key);
    if (manifest) {
        return do_install_impl(manifest, progress);
    }

    return {-1, "Could not download manifest"};
}

auto apps_t::queue_sideload(std::string manifest_string) //
    -> jobs::id_t
{
    auto job =
        jobs::job_t{std::bind(&apps_t::do_sideload, this, std::move(manifest_string), std::placeholders::_1)};

    return _jobs_api->append(std::move(job), "Sideloading App");
}
auto apps_t::do_sideload_sync(std::string manifest_string) //
    -> result_t
{
    auto _ = jobs::progress_t{};
    return do_sideload(std::move(manifest_string), _);
}
auto apps_t::do_sideload(std::string manifest_string, jobs::progress_t& progress) //
    -> result_t
{
    const auto [manifest, _] = _manifests_api->add_from_string(manifest_string);
    // Step 1: Validate transferred manifest
    if (manifest) {
        // Step 2: Forward to manifest installation
        return do_install_impl(manifest, progress);
    }

    return {-1, "Could not parse manifest"};
}

auto apps_t::do_install_impl(
    std::shared_ptr<app_manifest_t> manifest,
    jobs::progress_t& progress) //
    -> result_t
{
    progress.next_step("Loading manifest");

    // Step 1: Create app from manifest
    auto tmp = apps::app_t{apps::key_t{manifest->app(), manifest->version()}, manifest};
    if (!tmp.key().is_valid()) {
        return {-1, "Could not open app manifest"};
    }
    tmp.desired(apps::status_e::Installed);
    tmp.status(apps::status_e::ManifestDownloaded);

    progress.next_step("Acquiring download token");

    // Step 2: Determine current App status to decide where to continue
    auto app = _parent->query(tmp.key());
    if (!app) {
        _apps.push_back(std::make_shared<apps::app_t>(std::move(tmp)));
        app = *_apps.rbegin();
    }

    auto token = std::optional<Token>{};

    auto deployment_api = std::dynamic_pointer_cast<module::deployments_t>(api::query_module("deployments"));
    auto deployment = deployment_api->query_deployment("docker");

    switch (app->status()) {
        case apps::status_e::ManifestDownloaded: {
            progress.next_step("Acquiring download token");
            try {
                token = acquire_download_token(app->key().name(), app->key().version());
            } catch (const rust::Error& e) {
                progress.result(0, std::string{"Could not acquire download token: "} + e.what());
            }

            [[fallthrough]];
        }
        case apps::status_e::TokenAcquired: {
            // Step 4: Download App through deployment
            progress.next_step("Downloading App");
            const auto [res, message] = deployment->download_app(app, token);
            if (res != 0) {
                _parent->save();
                return {res, message};
            }
            app->status(apps::status_e::ImageDownloaded);
            [[fallthrough]];
        }
        case apps::status_e::ImageDownloaded: {
            progress.next_step("Expiring download token");
            const auto app_size = deployment->determine_app_size(app);
            app->installed_size(app_size.value_or(0));
            app->status(apps::status_e::Installed);
            break;
        }
        default: {
        }
    }

    // Final step: Persist successful installation into db
    _parent->save();
    return {0, {}};
}

auto apps_t::queue_uninstall(apps::key_t app_key) //
    -> jobs::id_t
{
    auto desc = "Uninstallation of "s + to_string(app_key);

    auto job = jobs::job_t{std::bind(&apps_t::do_uninstall, this, std::move(app_key), std::placeholders::_1)};

    return _jobs_api->append(std::move(job), std::move(desc));
}
auto apps_t::do_uninstall_sync(apps::key_t app_key) //
    -> result_t
{
    auto _ = jobs::progress_t{};
    return do_uninstall(std::move(app_key), _);
}
auto apps_t::do_uninstall(apps::key_t app_key, jobs::progress_t& progress) //
    -> result_t
{
    progress.num_steps(4);
    progress.next_step("Loading App manifest");

    // Step 1: Ensure App is actually installed
    auto app = _parent->query(app_key);
    if (!app) {
        return {-1, "Cannot uninstall "s + to_string(app_key) + ", which is not installed"};
    }

    // Step 2: Load App manifest
    auto manifest = app->manifest();

    app->desired(apps::status_e::NotInstalled);

    progress.next_step("Removing App instances");

    // Step 3: Stop and delete all instances of the App
    const auto instance_ids = _instances_api->instance_ids(app_key);
    for (const auto id : instance_ids) {
        _instances_api->remove(id);
    }

    // Step 4: Remove Docker image of the App
    progress.next_step("Removing App image");

    if (manifest) {
        const auto image = manifest->image_with_tag();
        auto docker_process = process_t{};
        docker_process.spawnp("docker", "rmi", "-f", image);
        docker_process.wait(false, true);
        if (docker_process.exit_code() != 0) {
            std::fprintf(
                stderr,
                "Warning: Could not remove image %s of app %s (%s)\n",
                image.c_str(),
                app_key.name().data(),
                app_key.version().data());
        }
    }

    // Step 5: Persist removal of App into db
    _apps.erase(
        std::remove_if(
            _apps.begin(),
            _apps.end(),
            [&app_key](const std::shared_ptr<apps::app_t>& elem) { return elem->key() == app_key; }),
        _apps.end());
    _parent->save();

    // Step 6: Remove App manifest
    progress.next_step("Removing App manifest");

    _manifests_api->erase(app_key);

    return {0, {}};
}

auto apps_t::queue_export_to(apps::key_t app_key, fs::path dest_dir) const //
    -> jobs::id_t
{
    auto job = jobs::job_t{std::bind(
        &apps_t::do_export_to,
        this,
        std::move(app_key),
        std::move(dest_dir),
        std::placeholders::_1)};

    return _jobs_api->append(std::move(job), "Exporting App " + to_string(app_key));
}
auto apps_t::do_export_to_sync(apps::key_t app_key, fs::path dest_dir) const //
    -> result_t
{
    auto _ = jobs::progress_t{};
    return do_export_to(std::move(app_key), std::move(dest_dir), _);
}
auto apps_t::do_export_to(apps::key_t app_key, fs::path dest_dir, jobs::progress_t& progress) const //
    -> result_t
{
    progress.num_steps(3);

    // Step 1: Load App manifest
    progress.next_step("Loading Manifest");
    auto app = _parent->query(app_key);
    auto manifest = app->manifest();
    if (!manifest) {
        return {-1, "App not connected to a Manifest"};
    }

    // Step 2: Create export directory
    progress.next_step("Creating export directory");
    auto ec = std::error_code{};
    fs::create_directories(dest_dir);
    if (ec) {
        return {-1, "Could not create export directory "s + dest_dir.c_str()};
    }

    // Step 3: Export image
    progress.next_step("Exporting App");
    auto docker_process = process_t{};
    const auto filename = dest_dir / (app_key.name().data() + "_"s + app_key.version().data() + ".tar");
    docker_process.spawnp("docker", "save", "--output", filename.string(), manifest->image_with_tag());
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0) {
        return {-1, docker_process.stderr()};
    }

    // Step 4: Copy manifest
    progress.next_step("Exporting Manifest");
    const auto manifest_src = _manifests_api->path(app_key);
    const auto manifest_dst = dest_dir / (app_key.name().data() + "_"s + app_key.version().data() + ".json");
    fs::copy_file(manifest_src, manifest_dst, ec);
    if (ec) {
        return {-1, "Could not copy Manifest"};
    }

    return {0, {}};
}

auto apps_t::queue_import_from(apps::key_t app_key, fs::path src_dir) //
    -> jobs::id_t
{
    auto job = jobs::job_t{std::bind(
        &apps_t::do_import_from,
        this,
        std::move(app_key),
        std::move(src_dir),
        std::placeholders::_1)};

    return _jobs_api->append(std::move(job), "Exporting App " + to_string(app_key));
}
auto apps_t::do_import_from_sync(apps::key_t app_key, fs::path src_dir) //
    -> result_t
{
    auto _ = jobs::progress_t{};
    return do_import_from(std::move(app_key), std::move(src_dir), _);
}
auto apps_t::do_import_from(apps::key_t app_key, fs::path src_dir, jobs::progress_t& /*progress*/) //
    -> result_t
{
    /* add App manifest */
    auto path = src_dir / (app_key.name().data() + "_"s + app_key.version().data() + ".json");
    const auto [manifest, _] = _manifests_api->add_from_file(path);
    if (!manifest) {
        return {-1, "Could not add App manifest"};
    }

    /* import image */
    path.replace_extension(".tar");
    auto docker_process = process_t{};
    docker_process.spawnp("docker", "load", "--input", path.c_str());
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0) {
        return {-1, docker_process.stderr()};
    }

    /* add to installed Apps */
    auto app = _parent->query(app_key);
    if (!app) {
        auto tmp = apps::app_t{app_key, manifest};
        _apps.push_back(std::make_shared<apps::app_t>(std::move(tmp)));
        app = *_apps.rbegin();
    }
    app->status(apps::status_e::Installed);
    app->desired(apps::status_e::Installed);

    return {0, {}};
}

auto apps_t::do_query(const apps::key_t& app_key) const noexcept //
    -> std::shared_ptr<apps::app_t>
{
    auto it =
        std::find_if(_apps.cbegin(), _apps.cend(), [&app_key](const std::shared_ptr<apps::app_t>& elem) {
            return elem->key() == app_key;
        });

    return it == _apps.cend() ? nullptr : *it;
}

auto apps_t::do_is_installed(const apps::key_t& app_key) const noexcept //
    -> bool
{
    auto app = _parent->query(app_key);

    return app ? (app->status() == apps::status_e::Installed) : false;
}

} // namespace impl
} // namespace module
} // namespace flecs
