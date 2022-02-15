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

#ifndef FLECS_daemon_modules_marketplace_h
#define FLECS_daemon_modules_marketplace_h

#include <string>
#include <utility>

#include "module_base/module.h"

namespace FLECS {

class module_marketplace_t : public module_t
{
private:
    module_error_e do_process(int argc, char** argv) override;

    using username_t = std::string;
    using token_t = std::string;
    std::pair<username_t, token_t> _auth;
};

} // namespace FLECS

#endif // FLECS_daemon_modules_marketplace_h
