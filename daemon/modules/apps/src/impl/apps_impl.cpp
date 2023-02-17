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

#include "impl/apps_impl.h"

#include <cpr/cpr.h>

#include <algorithm>
#include <fstream>

#include "api/api.h"
#include "common/app/manifest/manifest.h"
#include "common/instance/instance.h"
#include "modules/factory/factory.h"
#include "modules/instances/instances.h"
#include "modules/jobs/jobs.h"
#include "modules/manifests/manifests.h"
#include "modules/marketplace/marketplace.h"
#include "util/fs/fs.h"
#include "util/json/json.h"
#include "util/process/process.h"
#include "util/string/string_utils.h"

using std::operator""s;

namespace FLECS {
namespace impl {

static auto acquire_download_token(std::string_view license_key) //
    -> std::string
{
    const auto mp_api = dynamic_cast<const module_marketplace_t*>(api::query_module("mp").get());
    if (!mp_api) {
        return "";
    }

    const auto wc_user_token = mp_api->token();

    auto post_json = json_t{};
    post_json["wc_user_token"] = wc_user_token;
    post_json["license_key"] = license_key;

#ifndef NDEBUG
    const auto url = cpr::Url{"https://marketplace.flecs.tech:8443/api/v1/app/download"};
#else
    const auto url = cpr::Url{"https://marketplace.flecs.tech/api/v1/app/download"};
#endif // NDEBUG

    const auto http_res = cpr::Post(
        url,
        cpr::Header{{"content-type", "application/json"}},
        cpr::Body{post_json.dump()});

    if (http_res.status_code != 200) {
        return "";
    }

    const auto response_json = parse_json(http_res.text);
    if (!is_valid_json(response_json)) {
        return "";
    }

    const auto success = response_json["success"].get<bool>();
    const auto user_token = response_json["user_token"].get<std::string>();
    const auto access_token = response_json["access_token"]["token"].get<std::string>();
    const auto uuid = response_json["access_token"]["uuid"].get<std::string>();

    if (!success || user_token.empty() || access_token.empty() || uuid.empty()) {
        return "";
    }

    return stringify_delim(';', user_token, access_token, uuid);
}

static auto expire_download_token(const std::string& user_token, const std::string& access_token) //
    -> bool
{
    auto post_json = json_t{};
    post_json["user_token"] = user_token;
    post_json["access_token"] = access_token;

#ifndef NDEBUG
    const auto url = cpr::Url{"https://marketplace.flecs.tech/api/v1/app/finish"};
#else
    const auto url = cpr::Url{"https://marketplace.flecs.tech/api/v1/app/finish"};
#endif // NDEBUG

    const auto http_res = cpr::Post(
        url,
        cpr::Header{{"content-type", "application/json"}},
        cpr::Body{post_json.dump()});

    if (http_res.status_code != 200) {
        return false;
    }

    const auto response_json = parse_json(http_res.text);
    if (!is_valid_json(response_json)) {
        return false;
    }

    return response_json["success"].get<bool>();
}

module_apps_t::module_apps_t(FLECS::module_apps_t* parent)
    : _parent{parent}
    , _apps{}
    , _apps_mutex{}
    , _jobs_api{}
{}

module_apps_t::~module_apps_t()
{}

auto module_apps_t::do_init() //
    -> void
{
    _instances_api =
        std::dynamic_pointer_cast<FLECS::module_instances_t>(api::query_module("instances"));
    _jobs_api = std::dynamic_pointer_cast<FLECS::module_jobs_t>(api::query_module("jobs"));
    _manifests_api =
        std::dynamic_pointer_cast<FLECS::module_manifests_t>(api::query_module("manifests"));
    _manifests_api->base_path("/var/lib/flecs/manifests/");

    for (auto& app : _apps) {
        auto manifest = _manifests_api->query(app->key());
        if (manifest) {
            app->manifest(std::move(manifest));
        }
    }

    for (auto id : _instances_api->instance_ids()) {
        auto instance = _instances_api->query(id);
        instance->app(
            _parent->query(app_key_t{instance->app_name().data(), instance->app_version().data()}));
    }
}

auto module_apps_t::do_load(const fs::path& base_path) //
    -> void
{
    auto json_file = std::ifstream{base_path / "apps.json"};
    if (json_file.good()) {
        auto apps_json = parse_json(json_file);
        try {
            _apps.reserve(apps_json.size());
            for (const auto& app : apps_json) {
                _apps.push_back(std::make_shared<app_t>(app.get<app_t>()));
            }
        } catch (const std::exception& ex) {
            _apps.clear();
        }
    }
}

auto module_apps_t::do_save(const fs::path& base_path) const //
    -> void
{
    auto ec = std::error_code{};
    fs::create_directories(base_path, ec);
    if (ec) {
        return;
    }

    auto json_file =
        std::ofstream{base_path / "apps.json", std::ios_base::out | std::ios_base::trunc};
    auto apps_json = json_t::array();
    for (const auto& app : _apps) {
        apps_json.push_back(*app);
    }

    json_file << apps_json;
}

auto module_apps_t::do_list(const app_key_t& app_key) const //
    -> crow::response
{
    auto response = json_t::array();

    for (decltype(auto) app : _apps) {
        const auto apps_match = app_key.name().empty() || (app_key.name() == app->key().name());
        const auto versions_match = app_key.name().empty() || app_key.version().empty() ||
                                    (app_key.version() == app->key().version());
        if (apps_match && versions_match) {
            response.push_back(*app);
        }
    }

    return {crow::status::OK, "json", response.dump()};
}

auto module_apps_t::queue_install_from_marketplace(app_key_t app_key, std::string license_key) //
    -> crow::response
{
    auto job = job_t{std::bind(
        &module_apps_t::do_install_from_marketplace,
        this,
        std::move(app_key),
        std::move(license_key),
        std::placeholders::_1)};

    auto job_id = _jobs_api->append(std::move(job), "Installation of "s + to_string(app_key));
    return crow::response{
        crow::status::ACCEPTED,
        "json",
        "{\"jobId\":" + std::to_string(job_id) + "}"};
}

auto module_apps_t::do_install_from_marketplace(
    app_key_t app_key,
    std::string license_key,
    job_progress_t& progress) //
    -> result_t
{
    progress.num_steps(6);
    progress.next_step("Downloading manifest");

    // Download App manifest and forward to manifest installation, if download successful
    const auto [manifest, _] = _manifests_api->add_from_marketplace(app_key);
    if (manifest) {
        return do_install_impl(std::move(manifest), license_key, progress);
    }

    return {-1, "Could not download manifest"};
}

auto module_apps_t::queue_sideload(std::string manifest_string, std::string license_key) //
    -> crow::response
{
    auto job = job_t{std::bind(
        &module_apps_t::do_sideload,
        this,
        std::move(manifest_string),
        std::move(license_key),
        std::placeholders::_1)};

    auto job_id = _jobs_api->append(std::move(job), "Sideloading App");
    return crow::response{
        crow::status::ACCEPTED,
        "json",
        "{\"jobId\":" + std::to_string(job_id) + "}"};
}

auto module_apps_t::do_sideload(
    std::string manifest_string, std::string license_key, job_progress_t& progress) //
    -> result_t
{
    const auto [manifest, _] = _manifests_api->add_from_string(manifest_string);
    // Step 1: Validate transferred manifest
    if (manifest) {
        // Step 2: Forward to manifest installation
        return do_install_impl(std::move(manifest), license_key, progress);
    }

    return {-1, "Could not parse manifest"};
}

auto module_apps_t::do_install_impl(
    std::shared_ptr<app_manifest_t> manifest,
    std::string_view license_key,
    job_progress_t& progress) //
    -> result_t
{
    progress.next_step("Loading manifest");

    // Step 1: Create app from manifest
    auto tmp = app_t{app_key_t{manifest->app(), manifest->version()}, manifest};
    if (!tmp.key().is_valid()) {
        return {-1, "Could not open app manifest"};
    }
    tmp.desired(app_status_e::Installed);
    tmp.status(app_status_e::ManifestDownloaded);

    progress.desc("Installation of "s + manifest->title() + " (" + manifest->version() + ")");
    progress.next_step("Acquiring download token");

    // Step 2: Determine current App status to decide where to continue
    auto app = _parent->query(tmp.key());
    if (!app) {
        _apps.push_back(std::make_shared<app_t>(std::move(tmp)));
        app = *_apps.rbegin();
    }

    switch (app->status()) {
        case app_status_e::ManifestDownloaded: {
            progress.next_step("Acquiring download token");

            // Step 3: Acquire download token for App
            app->download_token(acquire_download_token(license_key));

            if (app->download_token().empty()) {
                progress.result(0, "Could not acquire download token");
            } else {
                app->status(app_status_e::TokenAcquired);
            }
            [[fallthrough]];
        }
        case app_status_e::TokenAcquired: {
            // Step 4: Pull Docker image for this App
            auto docker_login_process = process_t{};
            auto docker_pull_process = process_t{};
            auto docker_logout_process = process_t{};
            const auto args = split(app->download_token(), ';');

            if (args.size() == 3) {
                progress.next_step("Authenticating");

                auto login_attempts = 3;
                while (login_attempts-- > 0) {
                    docker_login_process = process_t{};
                    docker_login_process
                        .spawnp("docker", "login", "--username", "flecs", "--password", args[1]);
                    docker_login_process.wait(true, true);
                    if (docker_login_process.exit_code() == 0) {
                        break;
                    }
                }
            }

            if (docker_login_process.exit_code() != 0) {
                _parent->save();
                return {-1, docker_login_process.stderr()};
            }

            progress.next_step("Downloading App");

            auto pull_attempts = 3;
            while (pull_attempts-- > 0) {
                docker_pull_process = process_t{};
                docker_pull_process.spawnp("docker", "pull", manifest->image_with_tag());
                docker_pull_process.wait(true, true);
                if (docker_pull_process.exit_code() == 0) {
                    break;
                }
            }

            docker_logout_process.spawnp("docker", "logout");
            docker_logout_process.wait(true, true);

            if (docker_pull_process.exit_code() != 0) {
                _parent->save();
                return {-1, docker_pull_process.stderr()};
            }
            app->status(app_status_e::ImageDownloaded);
            [[fallthrough]];
        }
        case app_status_e::ImageDownloaded: {
            progress.next_step("Expiring download token");

            // Step 5: Expire download token
            const auto args = split(app->download_token(), ';');
            if (args.size() == 3) {
                const auto success = expire_download_token(args[0], args[2]);
                if (success) {
                    app->download_token("");
                    app->status(app_status_e::Installed);
                } else {
                    progress.result(0, "Could not expire download token");
                }
            } else {
                app->download_token("");
                app->status(app_status_e::Installed);
            }
            break;
        }
        default: {
        }
    }

    // Final step: Persist successful installation into db
    _parent->save();
    return {0, {}};
}

auto module_apps_t::queue_uninstall(app_key_t app_key, bool force) //
    -> crow::response
{
    if (!_parent->is_installed(app_key)) {
        auto response = json_t{};
        response["additionalInfo"] =
            "Cannot uninstall "s + to_string(app_key) + ", which is not installed";
        return {crow::status::BAD_REQUEST, "json", response.dump()};
    }

    auto job = job_t{std::bind(
        &module_apps_t::do_uninstall,
        this,
        std::move(app_key),
        std::move(force),
        std::placeholders::_1)};

    auto job_id = _jobs_api->append(std::move(job), "Uninstallation of "s + to_string(app_key));
    return crow::response{
        crow::status::ACCEPTED,
        "json",
        "{\"jobId\":" + std::to_string(job_id) + "}"};
}

auto module_apps_t::do_uninstall(app_key_t app_key, bool force, job_progress_t& progress) //
    -> result_t
{
    progress.num_steps(4);
    progress.next_step("Loading App manifest");

    // Step 1: Ensure App is actually installed
    if (!_parent->is_installed(app_key)) {
        return {-1, "Cannot uninstall "s + to_string(app_key) + ", which is not installed"};
    }

    // Step 2: Load App manifest
    auto app = _parent->query(app_key);
    auto manifest = app->manifest();

    progress.desc("Uninstallation of "s + manifest->title() + " (" + manifest->version() + ")");

    // Step 2a: Prevent removal of system apps
    if (cxx20::contains(manifest->category(), "system") && !force) {
        return {-1, "Not uninstalling system app "s + to_string(app_key)};
    }

    app->desired(app_status_e::NotInstalled);

    progress.next_step("Removing App instances");

    // Step 3: Stop and delete all instances of the App
    const auto instance_ids = _instances_api->instance_ids(app_key);
    for (const auto id : instance_ids) {
        _instances_api->remove(id);
    }

    // Step 4: Remove Docker image of the App
    progress.next_step("Removing App image");

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

    // Step 5: Persist removal of App into db
    _apps.erase(
        std::remove_if(
            _apps.begin(),
            _apps.end(),
            [&app_key](const std::shared_ptr<app_t>& elem) { return elem->key() == app_key; }),
        _apps.end());
    _parent->save();

    // Step 6: Remove App manifest
    progress.next_step("Removing App manifest");

    _manifests_api->erase(app_key);

    return {0, {}};
}

auto module_apps_t::queue_archive(app_key_t app_key) const //
    -> crow::response
{
    auto job = job_t{
        std::bind(&module_apps_t::do_archive, this, std::move(app_key), std::placeholders::_1)};

    auto job_id = _jobs_api->append(std::move(job), "Exporting App " + to_string(app_key));
    return crow::response{
        crow::status::ACCEPTED,
        "json",
        "{\"jobId\":" + std::to_string(job_id) + "}"};
}

auto module_apps_t::do_archive(app_key_t app_key, job_progress_t& /*progress*/) const //
    -> result_t
{
    // Step 1: Ensure App is actually installed
    if (!_parent->is_installed(app_key)) {
        return {-1, "Cannot export "s + to_string(app_key) + ", which is not installed"};
    }

    // Step 2: Load App manifest
    auto app = _parent->query(app_key);
    auto manifest = app->manifest();

    // Step 3: Create export directory
    auto ec = std::error_code{};
    fs::create_directories("/var/lib/flecs/export-tmp/apps/");
    if (ec) {
        return {-1, "Could not create export directory for "s + to_string(app_key)};
    }

    // Step 4: Export image
    auto docker_process = process_t{};
    const auto filename = std::string{"/var/lib/flecs/export-tmp/apps/"}
                              .append(app->key().name())
                              .append("_")
                              .append(app->key().version())
                              .append(".tar");
    docker_process.spawnp("docker", "save", "--output", filename, manifest->image_with_tag());
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0) {
        return {-1, docker_process.stderr()};
    }

    // Step 5: Copy manifest
    const auto manifest_src = fs::path{"/var/lib/flecs/apps"} / app_key.name().data() /
                              app_key.version().data() / "manifest.yml";
    const auto manifest_dst = fs::path{"/var/lib/flecs/export-tmp/apps/"} /
                              (app_key.name().data() + "_"s + app_key.version().data() + ".yml");
    fs::copy_file(manifest_src, manifest_dst, ec);
    if (ec) {
        return {-1, "Could not copy app manifest for "s + to_string(app_key)};
    }

    return {0, {}};
}

auto module_apps_t::do_contains(const app_key_t& app_key) const noexcept //
    -> bool
{
    return std::find_if(
               _apps.cbegin(),
               _apps.cend(),
               [&app_key](const std::shared_ptr<app_t>& elem) { return elem->key() == app_key; }) !=
           _apps.cend();
}

auto module_apps_t::do_query(const app_key_t& app_key) const noexcept //
    -> std::shared_ptr<app_t>
{
    auto it =
        std::find_if(_apps.cbegin(), _apps.cend(), [&app_key](const std::shared_ptr<app_t>& elem) {
            return elem->key() == app_key;
        });

    return it == _apps.cend() ? nullptr : *it;
}

auto module_apps_t::do_is_installed(const app_key_t& app_key) const noexcept //
    -> bool
{
    if (auto app = _parent->query(app_key)) {
        return app->status() == app_status_e::Installed;
    }

    return false;
}

} // namespace impl
} // namespace FLECS
