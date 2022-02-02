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

#ifndef FLECS_daemon_modules_app_manager_h
#define FLECS_daemon_modules_app_manager_h

#include <memory>

#include "db/app_db.h"
#include "modules/module.h"

namespace FLECS {

namespace Private {
class module_app_manager_private_t;
} // namespace Private

class module_app_manager_t : public module_t
{
public:
    module_app_manager_t();
    ~module_app_manager_t();

private:
    module_error_e do_process(int argc, char** argv) override;

    /*! Entry points for all commands - parse arguments and forward to implementation */
    module_error_e install(int argc, char** argv);
    module_error_e sideload(int argc, char** argv);
    module_error_e uninstall(int argc, char** argv);
    module_error_e create_instance(int argc, char** argv);
    module_error_e delete_instance(int argc, char** argv);
    module_error_e start_instance(int argc, char** argv);
    module_error_e stop_instance(int argc, char** argv);
    module_error_e list_apps(int argc, char** argv);
    module_error_e list_versions(int argc, char** argv);
    module_error_e list_instances(int argc, char** argv);

    std::unique_ptr<Private::module_app_manager_private_t> _impl;
};

} // namespace FLECS

#endif // FLECS_daemon_modules_app_manager_h
