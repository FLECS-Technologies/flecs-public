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

#ifndef FLECS_service_service_errors_h
#define FLECS_service_service_errors_h

#include <cstdint>

namespace FLECS {

enum service_error_e : std::uint32_t
{
    FLECS_OK,
    FLECS_FAILED,

    FLECS_ARGC,
    FLECS_USAGE,

    FLECS_IO,
    FLECS_IOFD,
    FLECS_IOR,
    FLECS_IOW,

    FLECS_APP_NOTINST,

    FLECS_INSTANCE_NOTEXIST,
    FLECS_INSTANCE_NOTRUN,
    FLECS_INSTANCE_APP,
    FLECS_INSTANCE_VERSION,

    FLECS_CURL = 0x80000000,
    FLECS_SQLITE = 0x81000000,
    FLECS_YAML = 0x82000000,
    FLECS_DOCKER = 0x83000000,
};

} // namespace FLECS

#endif // FLECS_service_service_errors_h
