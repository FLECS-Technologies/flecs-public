// Copyright 2021-2023 FLECS Technologies GmbH
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

#include <libdocker/libdocker.h>

namespace FLECS {

inline auto setup_libdocker_client() //
    -> docker::api_client_t
{
    return docker::api_client_t{
        docker::client::base_url_t{"unix:///run/docker.sock"},
        docker::client::timeout_t{std::chrono::milliseconds{10'000}},
        docker::client::api_version_e{docker::client::api_version_e::V1_41}};
}
} // namespace FLECS
