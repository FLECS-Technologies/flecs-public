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

#include "lib/libflecs.h"

#include <string>

#include "lib/private/libflecs_private.h"

namespace FLECS {

libflecs_t::libflecs_t()
    : _impl{new Private::libflecs_private_t{}}
{}

int libflecs_t::run_command(int argc, char** argv)
{
    auto strargs = std::string{};
    for (auto i = 0; i < argc; ++i)
    {
        strargs += argv[i];
        strargs += '\0';
    }
    return _impl->run_command(strargs);
}

} // namespace FLECS
