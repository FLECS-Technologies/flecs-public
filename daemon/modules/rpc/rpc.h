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

#ifndef FLECS_daemon_modules_rpc_h
#define FLECS_daemon_modules_rpc_h

#include "module_base/module.h"
#include "util/cxx20/string.h"

namespace FLECS {

class module_rpc_t : public module_t
{
public:
private:
    module_error_e do_process(int argc, char** argv) override;

    std::string _action;
    std::string _callee;
    std::string _method;
    // std::list<FLECS::any> _args;
};

} // namespace FLECS

#endif // FLECS_daemon_modules_rpc_h
