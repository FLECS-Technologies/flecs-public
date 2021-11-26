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

#ifndef FLECS_service_service_app_manager_h
#define FLECS_service_service_app_manager_h

#include "service/service.h"

#include <string>

namespace FLECS {

class service_app_manager : public service_t
{
public:
    service_app_manager();
    ~service_app_manager();

private:
    int do_process(int argc, char** argv) override;

    int install(int argc, char** argv);
    int uninstall(int argc, char** argv);
    int create_instance(int argc, char** argv);
    int delete_instance(int argc, char** argv);
    int start_instance(int argc, char** argv);
    int stop_instance(int argc, char** argv);
    int list_apps(int argc, char** argv);
    int list_instances(int argc, char** argv);

    int do_install(const std::string& app_name, const std::string& version);
    int do_uninstall(const std::string& app_name, const std::string& version);
    int do_create_instance(const std::string& app_name, const std::string& version, const std::string& description);
    int do_delete_instance(const std::string& id);
    int do_start_instance(const std::string& id);
    int do_stop_instance(const std::string& id);
    int do_list_apps();
    int do_list_instances();

    std::string build_manifest_url(const std::string& app_name, const std::string& version) const;
    std::string build_manifest_path(const std::string& app_name, const std::string& version) const;

    std::string _action;
};

} // namespace FLECS

#endif // FLECS_service_service_app_manager_h
