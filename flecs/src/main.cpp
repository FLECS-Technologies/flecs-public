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

#include <filesystem>

#include "flecs/api/api.h"
#include "flecs/modules/apps/apps.h"
#include "flecs/modules/data_layer/data_layer.h"
#include "flecs/modules/deployments/deployments.h"
#include "flecs/modules/factory/factory.h"
#include "flecs/modules/flecsport/flecsport.h"
#include "flecs/modules/floxy/floxy.h"
#include "flecs/modules/instances/instances.h"
#include "flecs/modules/jobs/jobs.h"
#include "flecs/modules/manifests/manifests.h"
#include "flecs/modules/system/system.h"
#include "flecs/modules/version/version.h"
#include "flecs/util/signal_handler/signal_handler.h"
#include "flecs_core_cxx_bridge/src/lib.rs.h"
#include "rust/cxx.h"

flecs::module::register_module_t<flecs::module::apps_t> _reg_apps("apps");
flecs::module::register_module_t<flecs::module::data_layer_t> _reg_data_layer("data-layer");
flecs::module::register_module_t<flecs::module::deployments_t> _reg_deployments("deployments");
flecs::module::register_module_t<flecs::module::flecsport_t> _reg_flecsport("flecsport");
flecs::module::register_module_t<flecs::module::floxy_t> _reg_floxy("floxy");
flecs::module::register_module_t<flecs::module::instances_t> _reg_instances("instances");
flecs::module::register_module_t<flecs::module::jobs_t> _reg_jobs("jobs");
flecs::module::register_module_t<flecs::module::manifests_t> _reg_manifests("manifests");
flecs::module::register_module_t<flecs::module::system_t> _reg_system("system");
flecs::module::register_module_t<flecs::module::version_t> _reg_version("version");

int main(int /*argc*/, char** /*argv*/)
{
    const auto local_socket_path = std::filesystem::path{"/run/flecs/flecsd.sock"};
    std::filesystem::remove(local_socket_path);
    std::filesystem::create_directories(local_socket_path.parent_path());

    flecs::api::init_modules();
    auto res =
        flecs::flecs_api_t::instance().app().multithreaded().local_socket_path(local_socket_path).run_async();
    flecs::flecs_api_t::instance().app().wait_for_server_start();

    std::filesystem::permissions(
        local_socket_path,
        std::filesystem::perms::group_write | std::filesystem::perms::others_write);

    start_server();

    res.get();

    stop_server();

    flecs::g_stop = true;

    flecs::api::deinit_modules();

    return 0;
}
