// Copyright 2021 FLECS Technologies GmbH
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

#include "service/service_app_manager.h"

#include "service/private/service_app_manager_private.h"

namespace FLECS {

service_app_manager_t::service_app_manager_t()
    : _impl{new Private::service_app_manager_private_t}
{}

service_app_manager_t::~service_app_manager_t()
{}

service_error_e service_app_manager_t::do_process(int argc, char** argv)
{
    if (argc < 1)
    {
        return FLECS_ARGC;
    }
    const auto action = argv[0];

    using action_callback_t = service_error_e (service_app_manager_t::*)(int, char**);
    using action_callback_table_t = FLECS::map_c<const char*, action_callback_t, 10, string_comparator>;
    constexpr action_callback_table_t action_callbacks = {{
        std::make_pair("install", &service_app_manager_t::install),
        std::make_pair("sideload", &service_app_manager_t::sideload),
        std::make_pair("uninstall", &service_app_manager_t::uninstall),
        std::make_pair("create-instance", &service_app_manager_t::create_instance),
        std::make_pair("delete-instance", &service_app_manager_t::delete_instance),
        std::make_pair("start-instance", &service_app_manager_t::start_instance),
        std::make_pair("stop-instance", &service_app_manager_t::stop_instance),
        std::make_pair("list-apps", &service_app_manager_t::list_apps),
        std::make_pair("list-versions", &service_app_manager_t::list_versions),
        std::make_pair("list-instances", &service_app_manager_t::list_instances),
    }};

    const auto it = action_callbacks.find(action);
    if (it != action_callbacks.end())
    {
        return std::invoke(it->second, this, argc - 1, &argv[1]);
    }

    return FLECS_USAGE;
}

service_error_e service_app_manager_t::install(int argc, char** argv)
{
    REQUIRED_ARGUMENT(app_name, 0);
    REQUIRED_ARGUMENT(version, 1);
    return _impl->do_install(app_name, version);
}

service_error_e service_app_manager_t::sideload(int argc, char** argv)
{
    REQUIRED_ARGUMENT(manifest, 0);
    return _impl->do_sideload(manifest);
}

service_error_e service_app_manager_t::uninstall(int argc, char** argv)
{
    REQUIRED_ARGUMENT(app_name, 0);
    REQUIRED_ARGUMENT(version, 1);
    return _impl->do_uninstall(app_name, version);
}

service_error_e service_app_manager_t::create_instance(int argc, char** argv)
{
    REQUIRED_ARGUMENT(app_name, 0);
    REQUIRED_ARGUMENT(version, 1);
    OPTIONAL_ARGUMENT(description, 2);
    return _impl->do_create_instance(app_name, version, description);
}

service_error_e service_app_manager_t::delete_instance(int argc, char** argv)
{
    REQUIRED_ARGUMENT(id, 0);
    OPTIONAL_ARGUMENT(app_name, 1);
    OPTIONAL_ARGUMENT(version, 2);
    return _impl->do_delete_instance(app_name, version, id);
}

service_error_e service_app_manager_t::start_instance(int argc, char** argv)
{
    REQUIRED_ARGUMENT(id, 0);
    OPTIONAL_ARGUMENT(app_name, 1);
    OPTIONAL_ARGUMENT(version, 2);
    return _impl->do_start_instance(app_name, version, id);
}

service_error_e service_app_manager_t::stop_instance(int argc, char** argv)
{
    REQUIRED_ARGUMENT(id, 0);
    OPTIONAL_ARGUMENT(app_name, 1);
    OPTIONAL_ARGUMENT(version, 2);
    return _impl->do_stop_instance(app_name, version, id);
}

service_error_e service_app_manager_t::list_apps(int /*argc*/, char** /*argv*/)
{
    return _impl->do_list_apps("");
}

service_error_e service_app_manager_t::list_versions(int argc, char** argv)
{
    REQUIRED_ARGUMENT(app_name, 0);
    return _impl->do_list_apps(app_name);
}

service_error_e service_app_manager_t::list_instances(int argc, char** argv)
{
    REQUIRED_ARGUMENT(app_name, 0);
    OPTIONAL_ARGUMENT(version, 1);
    return _impl->do_list_instances(app_name, version);
}

} // namespace FLECS
