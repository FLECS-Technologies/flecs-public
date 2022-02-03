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

#ifndef FLECS_daemon_modules_module_h
#define FLECS_daemon_modules_module_h

#include "errors.h"

namespace FLECS {

#define REQUIRED_ARGUMENT(arg, pos) \
    if (argc < (pos + 1))           \
    {                               \
        return FLECS_ARGC;          \
    }                               \
    const auto arg = argv[pos]

#define OPTIONAL_ARGUMENT(arg, pos) const auto arg = (argc > pos) ? argv[pos] : ""

class module_t
{
public:
    module_error_e process(int argc, char** argv);

protected:
    module_t() = default;
    virtual ~module_t() = default;

private:
    virtual module_error_e do_process(int argc, char** argv) = 0;

    bool _json_output;
};

} // namespace FLECS

#endif // FLECS_daemon_modules_module_h
