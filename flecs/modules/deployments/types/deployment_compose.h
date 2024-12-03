// Copyright 2024, FLECS Technologies GmbH
// SPDX-License-Identifier: Apache License 2.0

#pragma once

#include "deployment_docker.h"

namespace flecs {
namespace deployments {

class compose_t : public docker_t
{
public:
    compose_t();

    ~compose_t() override;

private:
    auto do_deployment_id() const noexcept //
        -> std::string_view override;

    auto do_download_app(std::shared_ptr<apps::app_t> app, std::optional<Token> token) //
        -> result_t override;
    auto do_delete_app(std::shared_ptr<apps::app_t> app) //
        -> result_t override;
    auto do_import_app(std::shared_ptr<apps::app_t> app, fs::path archive) //
        -> result_t override;
    auto do_export_app(std::shared_ptr<const apps::app_t> app, fs::path archive) //
        -> result_t override;

    auto do_import_volume(
        std::shared_ptr<instances::instance_t> instance,
        volume_t& volume,
        fs::path src_dir) //
        -> result_t override;
    auto do_import_volumes(
        std::shared_ptr<instances::instance_t> instance,
        fs::path src_dir) //
        -> result_t override;
    auto do_export_volume(
        std::shared_ptr<instances::instance_t> instance,
        const volume_t& volume,
        fs::path dest_dir) const //
        -> result_t override;
    auto do_export_volumes(
        std::shared_ptr<instances::instance_t> instance,
        fs::path dest_dir) const //
        -> result_t override;

    auto do_start_instance(std::shared_ptr<instances::instance_t> instance) //
        -> result_t override;
    auto do_stop_instance(std::shared_ptr<instances::instance_t> instance) //
        -> result_t override;
};

} // namespace deployments
} // namespace flecs
