// Copyright 2024, FLECS Technologies GmbH
// SPDX-License-Identifier: Apache License 2.0

#include "flecs/modules/deployments/types/deployment_compose.h"

#include "flecs/common/app/manifest/manifest.h"
#include "flecs/modules/apps/types/app.h"
#include "flecs/util/process/process.h"

namespace flecs {
namespace deployments {

auto compose_t::do_deployment_id() const noexcept //
    -> std::string_view
{
    return "compose";
}

auto compose_t::do_download_app(std::shared_ptr<apps::app_t> app, std::optional<Token> token) //
    -> result_t
{
    if (token.has_value() && !token->username.empty() && !token->password.empty()) {
        const auto [res, message] = docker_login(std::move(token));
        if (res != 0) {
            std::fprintf(stderr, "Warning: docker login unsuccessful: %s\n", message.c_str());
        }
    }

    const auto deployment = app->manifest()->deployment();
    if (!deployment.contains("compose") || !deployment.at("compose").contains("yaml")) {
        return {-1, "App manifest does not contain compose yaml"};
    }
    auto compose_json = app->manifest()->deployment().at("compose").at("yaml");
    if (compose_json.contains("networks")) {
        for (const auto& [network, properties] : compose_json.at("networks").items()) {
            if (network == "flecs") {
                return {-1, "Invalid App manifest: network name 'flecs' is reserved"};
            }
        }
    }

    auto pull_process = process_t{};
    auto pull_attempts = 3;
    while (pull_attempts-- > 0) {
        pull_process.stdin(compose_json.dump());
        pull_process.spawnp("docker-compose", "-f", "-", "pull");
        pull_process.wait(true, true);
        if (pull_process.exit_code() == 0) {
            break;
        }
    }

    if (token.has_value()) {
        auto process = process_t{};
        process.spawnp("docker", "logout");
        process.wait(true, true);
    }

    if (pull_process.exit_code() != 0) {
        return {-1, pull_process.stderr()};
    }

    return {0, {}};
}

auto compose_t::do_delete_app(std::shared_ptr<apps::app_t> app) //
    -> result_t
{
    auto manifest = app->manifest();
    if (!manifest) {
        return {-1, "Could not access app manifest"};
    }

    const auto deployment = app->manifest()->deployment();
    if (!deployment.contains("compose") || !deployment.at("compose").contains("yaml") ||
        !deployment.at("compose").at("yaml").contains("services")) {
        return {-1, "App manifest does not contain compose yaml"};
    }
    auto compose_json = app->manifest()->deployment().at("compose").at("yaml");

    auto [res, message] = result_t{};
    for (const auto& [service, properties] : compose_json.at("services").items()) {
        if (properties.contains("image")) {
            auto docker_process = process_t{};
            docker_process.spawnp("docker", "rmi", "-f", properties.at("image").get<std::string>());
            docker_process.wait(false, true);
            if (docker_process.exit_code() != 0) {
                res = -1;
                message.append(docker_process.stderr());
            }
        }
    }

    return {res, message};
}

auto compose_t::do_import_app(std::shared_ptr<apps::app_t> app, fs::path archive) //
    -> result_t
{
    const auto manifest = app->manifest();
    if (!manifest) {
        return {-1, "Could not access app manifest"};
    }

    const auto deployment = app->manifest()->deployment();
    if (!deployment.contains("compose") || !deployment.at("compose").contains("yaml") ||
        !deployment.at("compose").at("yaml").contains("services")) {
        return {-1, "App manifest does not contain a valid compose yaml"};
    }

    auto compose_services = app->manifest()->deployment().at("compose").at("yaml").at("services");
    auto services = std::vector<std::string>{};
    for (const auto& [service, _] : compose_services.items()) {
        services.emplace_back(service);
    }

    auto [res, message] = result_t{};
    for (const auto& service : services) {
        auto service_archive = archive;
        service_archive.replace_extension(service + ".tar");
        const auto [local_res, local_message] = docker_load(std::move(service_archive));
        if (local_res != 0) {
            res = local_res;
        }
        message.append(local_message);
    }

    return {res, message};
}

auto compose_t::do_export_app(std::shared_ptr<const apps::app_t> app, fs::path archive) //
    -> result_t
{
    const auto manifest = app->manifest();
    if (!manifest) {
        return {-1, "Could not access app manifest"};
    }

    const auto deployment = app->manifest()->deployment();
    if (!deployment.contains("compose") || !deployment.at("compose").contains("yaml") ||
        !deployment.at("compose").at("yaml").contains("services")) {
        return {-1, "App manifest does not contain a valid compose yaml"};
    }

    auto compose_services = app->manifest()->deployment().at("compose").at("yaml").at("services");
    auto images = std::vector<std::tuple<std::string, std::string>>{};
    for (const auto& [service, properties] : compose_services.items()) {
        if (properties.contains("image")) {
            images.emplace_back(std::make_tuple(properties.at("image").get<std::string>(), service));
        }
    }

    auto [res, message] = result_t{};
    for (const auto& [image, service] : images) {
        auto service_archive = archive;
        service_archive.replace_extension(service + ".tar");
        const auto [local_res, local_message] = docker_save(image, service_archive);
        if (local_res != 0) {
            res = local_res;
        }
        message.append(local_message);
    }

    return {res, message};
}

auto compose_t::do_start_instance(std::shared_ptr<instances::instance_t> instance) //
    -> result_t
{
    auto app = instance->app();
    if (!app) {
        return {-1, "Instance not connected to an app"};
    }

    auto manifest = app->manifest();
    if (!manifest) {
        return {-1, "Could not access app manifest"};
    }

    const auto deployment = app->manifest()->deployment();
    if (!deployment.contains("compose") || !deployment.at("compose").contains("yaml")) {
        return {-1, "App manifest does not contain compose yaml"};
    }
    auto compose_json = app->manifest()->deployment().at("compose").at("yaml").dump();

    const auto project_name = std::string{"flecs-"} + instance->id().hex();

    /* Create containers */
    auto compose_process = process_t{};
    compose_process.stdin(compose_json);
    {
        const auto res = compose_process.spawnp("docker-compose", "-p", project_name, "-f", "-", "create");
        if (res < 0) {
            return {res, {}};
        }
    }
    compose_process.wait(false, true);
    if (compose_process.exit_code() != 0) {
        return {-1, compose_process.stderr()};
    }

    /* Start containers */
    compose_process = process_t{};
    compose_process.stdin(compose_json);
    {
        const auto res = compose_process.spawnp("docker-compose", "-p", project_name, "-f", "-", "start");
        if (res < 0) {
            return {res, {}};
        }
    }
    compose_process.wait(false, true);
    if (compose_process.exit_code() != 0) {
        return {-1, compose_process.stderr()};
    }

    return {0, {}};
}

auto compose_t::do_stop_instance(std::shared_ptr<instances::instance_t> instance) //
    -> result_t
{
    auto app = instance->app();
    if (!app) {
        return {-1, "Instance not connected to an app"};
    }

    auto manifest = app->manifest();
    if (!manifest) {
        return {-1, "Could not access app manifest"};
    }

    const auto deployment = app->manifest()->deployment();
    if (!deployment.contains("compose") || !deployment.at("compose").contains("yaml")) {
        return {-1, "App manifest does not contain compose yaml"};
    }
    auto compose_json = app->manifest()->deployment().at("compose").at("yaml").dump();

    const auto project_name = std::string{"flecs-"} + instance->id().hex();
    /* Stop containers */
    auto compose_process = process_t{};
    compose_process.stdin(compose_json);
    const auto res = compose_process.spawnp("docker-compose", "-p", project_name, "-f", "-", "stop");
    if (res < 0) {
        return {res, {}};
    }
    compose_process.wait(true, true);
    if (compose_process.exit_code() != 0) {
        return {-1, compose_process.stderr()};
    }

    /* Remove containers */
    compose_process = process_t{};
    compose_process.stdin(compose_json);
    {
        const auto res = compose_process.spawnp("docker-compose", "-p", project_name, "-f", "-", "rm", "-f");
        if (res < 0) {
            return {res, {}};
        }
    }
    compose_process.wait(true, true);
    if (compose_process.exit_code() != 0) {
        return {-1, compose_process.stderr()};
    }

    return {0, {}};
}

auto compose_t::do_import_volume(
    std::shared_ptr<instances::instance_t> instance,
    volume_t& volume,
    fs::path dest_dir) //
    -> result_t
{
    const auto name = "flecs-" + instance->id().hex() + "_" + volume.host();
    return docker_import_volume(std::move(name), std::move(dest_dir));
}

auto compose_t::do_import_volumes(std::shared_ptr<instances::instance_t> instance, fs::path dest_dir) //
    -> result_t
{
    auto app = instance->app();
    if (!app) {
        return {-1, "Instance not connected to an app"};
    }
    auto manifest = app->manifest();
    if (!manifest) {
        return {-1, "Could not access app manifest"};
    }
    const auto deployment = manifest->deployment();
    if (!deployment.contains("compose") || !deployment.at("compose").contains("yaml")) {
        return {-1, "App manifest does not contain compose yaml"};
    }
    auto compose_json = app->manifest()->deployment().at("compose").at("yaml");
    if (!compose_json.contains("volumes")) {
        return {0, {}};
    }

    for (const auto& [volume, _] : compose_json.at("volumes").items()) {
        auto docker_volume = volume_t{volume + ":/tmp"};
        const auto [res, additional_info] = import_volume(instance, docker_volume, dest_dir);
        if (res != 0) {
            return {res, additional_info};
        }
    }

    return {0, {}};
}

auto compose_t::do_export_volume(
    std::shared_ptr<instances::instance_t> instance,
    const volume_t& volume,
    fs::path dest_dir) const //
    -> result_t
{
    const auto name = "flecs-" + instance->id().hex() + "_" + volume.host();
    return docker_export_volume(std::move(name), std::move(dest_dir));
}

auto compose_t::do_export_volumes(std::shared_ptr<instances::instance_t> instance, fs::path dest_dir) const //
    -> result_t
{
    auto app = instance->app();
    if (!app) {
        return {-1, "Instance not connected to an app"};
    }
    auto manifest = app->manifest();
    if (!manifest) {
        return {-1, "Could not access app manifest"};
    }
    const auto deployment = manifest->deployment();
    if (!deployment.contains("compose") || !deployment.at("compose").contains("yaml")) {
        return {-1, "App manifest does not contain compose yaml"};
    }
    auto compose_json = app->manifest()->deployment().at("compose").at("yaml");
    if (!compose_json.contains("volumes")) {
        return {0, {}};
    }

    for (const auto& [volume, _] : compose_json.at("volumes").items()) {
        auto docker_volume = volume_t{volume + ":/tmp"};
        const auto [res, additional_info] = export_volume(instance, docker_volume, dest_dir);
        if (res != 0) {
            return {res, additional_info};
        }
    }

    return {0, {}};
}

compose_t::compose_t() = default;

compose_t::~compose_t() = default;

} // namespace deployments
} // namespace flecs
