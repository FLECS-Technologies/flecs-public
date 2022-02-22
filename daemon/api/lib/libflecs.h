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

#ifndef CDA38A84_96B2_4ABF_BF57_39654CBCFB5D
#define CDA38A84_96B2_4ABF_BF57_39654CBCFB5D

#include <memory>
#include <string>
#include <utility>
#include <vector>

#include "util/string/string_utils.h"

namespace FLECS {

namespace Private {
class libflecs_private_t;
}

template <typename Impl = Private::libflecs_private_t>
class libflecs_t
{
public:
    FLECS_EXPORT libflecs_t();

    FLECS_EXPORT ~libflecs_t();

    FLECS_EXPORT int connect(const std::string& path);
    FLECS_EXPORT int connect(const std::string& host, int port);

    FLECS_EXPORT int disconnect();

    // app management
    FLECS_EXPORT int install_app(const std::string& app, const std::string& version, const std::string& license);
    FLECS_EXPORT int uninstall_app(const std::string& app, const std::string& version);
    FLECS_EXPORT int sideload_app(const std::string& manifest_path);
    FLECS_EXPORT int list_apps();
    FLECS_EXPORT int list_instances(const std::string& app, const std::string& version);
    FLECS_EXPORT int list_versions(const std::string& app);

    // instance management
    FLECS_EXPORT int create_instance(
        const std::string& app, const std::string& version, const std::string& instanceName);
    FLECS_EXPORT int delete_instance(const std::string& instanceId, const std::string& app, const std::string& version);
    FLECS_EXPORT int start_instance(const std::string& instanceId, const std::string& app, const std::string& version);
    FLECS_EXPORT int stop_instance(const std::string& instanceId, const std::string& app, const std::string& version);

    // system info
    FLECS_EXPORT int version();
    FLECS_EXPORT int ping();

    // string-based interface
    FLECS_EXPORT int run_command(const std::string& command, const std::vector<std::string>& args);
    FLECS_EXPORT int run_command(int argc, char** argv);

    // retrieve HTTP status code
    FLECS_EXPORT int response_code() const noexcept;

    // retrieve response as formatted JSON string
    FLECS_EXPORT std::string json_response() const noexcept;

private:
    std::unique_ptr<Impl> _impl;
};

extern template class FLECS_EXPORT libflecs_t<Private::libflecs_private_t>;

} // namespace FLECS

#endif // CDA38A84_96B2_4ABF_BF57_39654CBCFB5D
