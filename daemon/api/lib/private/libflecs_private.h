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

#ifndef FLECS_daemon_libflecs_private_h
#define FLECS_daemon_libflecs_private_h

#include <string>

namespace FLECS {
namespace Private {

class libflecs_private_t
{
public:
    FLECS_EXPORT int run_command(const std::string& args);

    FLECS_EXPORT std::string response() const { return _response; }

private:
    int _return_code;
    std::string _response;
};

} // namespace Private
} // namespace FLECS

#endif // FLECS_daemon_libflecs_private_h