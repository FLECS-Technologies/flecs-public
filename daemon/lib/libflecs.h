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

#ifndef FLECS_daemon_libflecs_h
#define FLECS_daemon_libflecs_h

#include <memory>

#include "private/libflecs_private.h"
#include "util/string/string_utils.h"

namespace FLECS {

namespace Private {
class libflecs_private_t;
}

class libflecs_t
{
public:
    FLECS_EXPORT libflecs_t();

    template <typename T, typename... Args>
    FLECS_EXPORT int run_command(T&& command, Args&&... args)
    {
        std::string strargs = stringify_delim('\0', "flecs", command, args...);
        return _impl->run_command(strargs);
    }

    FLECS_EXPORT int run_command(int argc, char** argv);

    FLECS_EXPORT std::string response() const;

private:
    std::unique_ptr<Private::libflecs_private_t> _impl;
};

std::string libflecs_t::response() const
{
    return _impl->response();
}

} // namespace FLECS

#endif // FLECS_daemon_libflecs_h
