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

#include "common/instance/instance_id.h"
#include "module_base/module.h"

namespace FLECS {

class app_key_t;
class instance_config_t;

namespace impl {
class module_instances_t;
} // namespace impl

class module_instances_t FLECS_FINAL_UNLESS_TESTED : public module_t
{
public:
    ~module_instances_t() override;

    /*! @brief Lists all available instances
     *
     * @param app_key (optional) limit list to all or specific version of specific App
     *
     * @return HTTP response
     */
    auto list(const app_key_t& app_key) const //
        -> crow::response;
    auto list(std::string app_name, std::string version) const //
        -> crow::response;
    auto list(std::string app_name) const //
        -> crow::response;
    auto list() const //
        -> crow::response;

    auto create(app_key_t app_key, std::string instance_name) //
        -> crow::response;
    auto create(app_key_t app_key) //
        -> crow::response;
    auto create(std::string app_name, std::string version, std::string instance_name) //
        -> crow::response;
    auto create(std::string app_name, std::string version) //
        -> crow::response;

    auto start(instance_id_t instance_id) //
        -> crow::response;

    auto stop(instance_id_t instance_id) //
        -> crow::response;

    auto remove(instance_id_t instance_id) //
        -> crow::response;

    auto get_config(instance_id_t instance_id) const //
        -> crow::response;

    auto post_config(instance_id_t instance_id, const instance_config_t& config) //
        -> crow::response;

    auto details(instance_id_t instance_id) const //
        -> crow::response;

    auto logs(instance_id_t instance_id) const //
        -> crow::response;

    auto update(instance_id_t instance_id, std::string from, std::string to) //
        -> crow::response;

    auto archive(instance_id_t instance_id) const //
        -> crow::response;

protected:
    friend class module_factory_t;

    module_instances_t();

    void do_init() override;
    void do_deinit() override {}

    std::unique_ptr<impl::module_instances_t> _impl;
};

} // namespace FLECS
