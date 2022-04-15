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

#include <cpr/cpr.h>

#include "app/app.h"
#include "factory/factory.h"
#include "marketplace/marketplace.h"
#include "private/app_manager_private.h"
#include "util/json/json_parser.h"
#include "util/process/process.h"

namespace FLECS {
namespace Private {

namespace {

std::string build_manifest_url(const std::string& app_name, const std::string& version)
{
#ifndef NDEBUG
    auto url = std::string{"https://marketplace.flecs.tech:8443/manifests/apps/"};
#else
    auto url = std::string{"https://marketplace.flecs.tech/manifests/apps/"};
#endif // NDEBUG

    url.append(app_name);
    url.append("/");
    url.append(version);
    url.append("/");
    url.append("manifest.yml");

    return url;
}

int download_manifest(const std::string& app_name, const std::string& version)
{
    const auto path = build_manifest_path(app_name, version);
    const auto manifest = fopen(path.c_str(), "w");
    if (manifest == nullptr)
    {
        std::fprintf(stderr, "Could not open %s for writing\n", path.c_str());
        return -1;
    }

    const auto url = build_manifest_url(app_name, version);
    auto response = cpr::Get(cpr::Url{url.c_str()});
    if (response.status_code != static_cast<long>(http_status_e::Ok))
    {
        std::fprintf(stderr, "Could not download app manifest: HTTP return code %ld\n", response.status_code);
        return -1;
    }
    const auto bytes_written = fwrite(response.text.data(), 1, response.text.length(), manifest);
    fclose(manifest);
    if (bytes_written != response.text.length())
    {
        std::fprintf(stderr, "Could not download app manifest: Write error %d\n", errno);
        return -1;
    }

    return 0;
}

std::string acquire_download_token(const std::string& license_key)
{
    const auto mp_api = dynamic_cast<const module_marketplace_t*>(api::query_module("mp").get());
    if (!mp_api)
    {
        return "";
    }

    const auto wc_user_token = mp_api->token();

    auto post_json = Json::Value{};
    post_json["wc_user_token"] = wc_user_token;
    post_json["license_key"] = license_key;

#ifndef NDEBUG
    const auto url = cpr::Url{"https://marketplace.flecs.tech:8443/api/v1/app/download"};
#else
    const auto url = cpr::Url{"https://marketplace.flecs.tech/api/v1/app/download"};
#endif // NDEBUG

    const auto http_res =
        cpr::Post(url, cpr::Header{{"content-type", "application/json"}}, cpr::Body{post_json.toStyledString()});

    if (http_res.status_code != 200)
    {
        return "";
    }

    const auto response_json = parse_json(http_res.text);
    if (!response_json.has_value())
    {
        return "";
    }

    const auto success = response_json.value()["success"].as<bool>();
    const auto user_token = response_json.value()["user_token"].as<std::string>();
    const auto access_token = response_json.value()["access_token"]["token"].as<std::string>();
    const auto uuid = response_json.value()["access_token"]["uuid"].as<std::string>();

    if (!success || user_token.empty() || access_token.empty() || uuid.empty())
    {
        return "";
    }

    return stringify_delim(';', user_token, access_token, uuid);
}

bool expire_download_token(const std::string& user_token, const std::string& access_token)
{
    auto post_json = Json::Value{};
    post_json["user_token"] = user_token;
    post_json["access_token"] = access_token;

#ifndef NDEBUG
    const auto url = cpr::Url{"https://marketplace.flecs.tech:8443/api/v1/app/finish"};
#else
    const auto url = cpr::Url{"https://marketplace.flecs.tech/api/v1/app/finish"};
#endif // NDEBUG

    const auto http_res =
        cpr::Post(url, cpr::Header{{"content-type", "application/json"}}, cpr::Body{post_json.toStyledString()});

    if (http_res.status_code != 200)
    {
        return false;
    }

    const auto response_json = parse_json(http_res.text);
    if (!response_json.has_value())
    {
        return false;
    }

    return response_json.value()["success"].as<std::string>() == "true";
}

} // namespace

http_status_e module_app_manager_private_t::do_install(
    const std::string& app_name, const std::string& version, const std::string& license_key, Json::Value& response)
{
    response["app"] = app_name;
    response["version"] = version;

    // Download app manifest and forward to manifest installation, if download successful
    const auto res = download_manifest(app_name, version);
    if (res != 0)
    {
        response["additionalInfo"] = "Could not download manifest for " + app_name + " (" + version + ")";
        return http_status_e::InternalServerError;
    };

    return do_install(build_manifest_path(app_name, version), license_key, response);
}

http_status_e module_app_manager_private_t::do_install(
    const std::string& manifest, const std::string& license_key, Json::Value& response)
{
    const auto desired = INSTALLED;

    // Step 1: Load app manifest
    const auto app = app_t{manifest};
    if (!app.yaml_loaded())
    {
        response["additionalInfo"] = "Could not open app manifest " + manifest;
        return http_status_e::InternalServerError;
    }
    response["additionalInfo"] = std::string{};
    response["app"] = app.name();
    response["version"] = app.version();

    // Step 2: Determine current app status to decide where to continue
    auto app_entry = _app_db.query_app({app.name(), app.version()})
                         .value_or(apps_table_entry_t{
                             apps_table_primary_t{app.name(), app.version()},
                             apps_table_data_t{MANIFEST_DOWNLOADED, desired, app.category(), 0, license_key, ""}});

    // Step 3: Add database entry for app. At this point the existence of the requested app is guaranteed as its
    // manifest was transferred to the local storage, so it is safe to add it to the local app database
    _app_db.insert_app(app_entry);

    switch (app_entry.status)
    {
        case MANIFEST_DOWNLOADED: {
            // Step 4: Acquire download token for app
            app_entry.download_token = acquire_download_token(license_key);

            if (app_entry.download_token.empty())
            {
                response["additionalInfo"] = "Could not acquire download token";
            }
            else
            {
                app_entry.status = TOKEN_ACQUIRED;
            }

            _app_db.insert_app(app_entry);
            [[fallthrough]];
        }
        case TOKEN_ACQUIRED: {
            // Step 5: Pull Docker image for this app
            auto docker_login_process = process_t{};
            auto docker_pull_process = process_t{};
            auto docker_logout_process = process_t{};
            const auto args = split(app_entry.download_token, ';');

            if (args.size() == 3)
            {
                auto login_attempts = 3;
                while (login_attempts-- > 0)
                {
                    docker_login_process = process_t{};
                    docker_login_process.spawnp("docker", "login", "--username", "flecs", "--password", args[1]);
                    docker_login_process.wait(true, true);
                    if (docker_login_process.exit_code() == 0)
                    {
                        break;
                    }
                }
            }

            if (docker_login_process.exit_code() != 0)
            {
                response["additionalInfo"] = docker_login_process.stderr();
                return http_status_e::InternalServerError;
            }

            auto pull_attempts = 3;
            while (pull_attempts-- > 0)
            {
                docker_pull_process = process_t{};
                docker_pull_process.spawnp("docker", "pull", app.image_with_tag());
                docker_pull_process.wait(true, true);
                if (docker_pull_process.exit_code() == 0)
                {
                    break;
                }
            }

            docker_logout_process.spawnp("docker", "logout");
            docker_logout_process.wait(true, true);

            if (docker_pull_process.exit_code() != 0)
            {
                response["additionalInfo"] = docker_pull_process.stderr();
                return http_status_e::InternalServerError;
            }
            app_entry.status = IMAGE_DOWNLOADED;
            _app_db.insert_app(app_entry);
            [[fallthrough]];
        }
        case IMAGE_DOWNLOADED: {
            // Step 6: Expire download token
            const auto args = split(app_entry.download_token, ';');
            if (args.size() == 3)
            {
                const auto success = expire_download_token(args[0], args[2]);
                if (success)
                {
                    app_entry.download_token.clear();
                    app_entry.status = INSTALLED;
                }
                else
                {
                    response["additionalInfo"] = "Could not expire download token";
                }
            }
            else
            {
                app_entry.download_token.clear();
                app_entry.status = INSTALLED;
            }
            _app_db.insert_app(app_entry);
            break;
        }
        default: {
        }
    }

    // Final step: Persist successful installation into db
    _app_db.persist();

    return http_status_e::Ok;
}

} // namespace Private
} // namespace FLECS
