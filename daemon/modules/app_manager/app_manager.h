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

#ifndef B9B56278_BFF8_4E56_83D7_1129F37256DF
#define B9B56278_BFF8_4E56_83D7_1129F37256DF

#include <memory>

#include "module_base/module.h"

namespace FLECS {

namespace Private {
class module_app_manager_private_t;
} // namespace Private

class module_app_manager_t : public module_t
{
public:
    ~module_app_manager_t() override;

protected:
    friend class module_factory_t;

    module_app_manager_t();

private:
    void do_init() override;

    /*! Endpoints - parse arguments and forward to implementation */
    http_status_e install(const nlohmann::json& args, nlohmann::json& response);
    http_status_e sideload(const nlohmann::json& args, nlohmann::json& response);
    http_status_e uninstall(const nlohmann::json& args, nlohmann::json& response);
    http_status_e create_instance(const nlohmann::json& args, nlohmann::json& response);
    http_status_e delete_instance(const nlohmann::json& args, nlohmann::json& response);
    http_status_e start_instance(const nlohmann::json& args, nlohmann::json& response);
    http_status_e stop_instance(const nlohmann::json& args, nlohmann::json& response);
    http_status_e instance_details(const nlohmann::json& args, nlohmann::json& response);
    http_status_e instance_log(const nlohmann::json& args, nlohmann::json& response);
    http_status_e list_apps(const nlohmann::json& args, nlohmann::json& response);
    http_status_e list_versions(const nlohmann::json& args, nlohmann::json& response);
    http_status_e list_instances(const nlohmann::json& args, nlohmann::json& response);

    std::unique_ptr<Private::module_app_manager_private_t> _impl;
};

} // namespace FLECS

#endif // B9B56278_BFF8_4E56_83D7_1129F37256DF
