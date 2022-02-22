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

#ifndef B9FC4679_2559_4145_830A_947F1BB2CF52
#define B9FC4679_2559_4145_830A_947F1BB2CF52

#include <cpr/cpr.h>

#include "libflecs.h"
#include "util/http/status_codes.h"

namespace Json {
class Value;
}

namespace FLECS {
namespace Private {

class libflecs_private_t
{
public:
    FLECS_EXPORT libflecs_private_t();

    FLECS_EXPORT ~libflecs_private_t();

    FLECS_EXPORT int do_connect(std::string path);
    FLECS_EXPORT int do_connect(std::string host, int port);

    FLECS_EXPORT int do_disconnect();

    // app management
    FLECS_EXPORT int do_install_app(const std::string& app, const std::string& version, const std::string& license);
    FLECS_EXPORT int do_uninstall_app(const std::string& app, const std::string& version);
    FLECS_EXPORT int do_sideload_app(const std::string& manifest_path);
    FLECS_EXPORT int do_list_apps();
    FLECS_EXPORT int do_list_instances(const std::string& app, const std::string& version);
    FLECS_EXPORT int do_list_versions(const std::string& app);

    // instance management
    FLECS_EXPORT int do_create_instance(
        const std::string& app, const std::string& version, const std::string& instanceName);
    FLECS_EXPORT int do_delete_instance(
        const std::string& instanceId, const std::string& app, const std::string& version);
    FLECS_EXPORT int do_start_instance(
        const std::string& instanceId, const std::string& app, const std::string& version);
    FLECS_EXPORT int do_stop_instance(
        const std::string& instanceId, const std::string& app, const std::string& version);

    // system info
    FLECS_EXPORT int do_version();
    FLECS_EXPORT int do_ping();

    // string-based interface
    FLECS_EXPORT int do_run_command(const std::string& command, const std::vector<std::string>& args);
    // string-based app management
    FLECS_EXPORT int dispatch_install_app(const std::vector<std::string>& args);
    FLECS_EXPORT int dispatch_uninstall_app(const std::vector<std::string>& args);
    FLECS_EXPORT int dispatch_sideload_app(const std::vector<std::string>& args);
    FLECS_EXPORT int dispatch_list_apps(const std::vector<std::string>& args);
    FLECS_EXPORT int dispatch_list_instances(const std::vector<std::string>& args);
    FLECS_EXPORT int dispatch_list_versions(const std::vector<std::string>& args);
    // sting-based instance management
    FLECS_EXPORT int dispatch_create_instance(const std::vector<std::string>& args);
    FLECS_EXPORT int dispatch_delete_instance(const std::vector<std::string>& args);
    FLECS_EXPORT int dispatch_start_instance(const std::vector<std::string>& args);
    FLECS_EXPORT int dispatch_stop_instance(const std::vector<std::string>& args);
    // string-based system management
    FLECS_EXPORT int dispatch_version(const std::vector<std::string>& args);
    FLECS_EXPORT int dispatch_ping(const std::vector<std::string>& args);

    // retrieve actual HTTP status code
    FLECS_EXPORT int do_response_code() const noexcept;

    // retrieve response as formatted JSON string
    FLECS_EXPORT std::string do_json_response() const noexcept;

private:
    cpr::Url build_url(const std::string& endpoint);
    int get(const std::string& endpoint);
    int post(const std::string& endpoint, const char* data);
    int put(const std::string& endpoint, const char* data);

    cpr::Url _base_url;
    std::string _path;
    cpr::Response _response;
};

} // namespace Private
} // namespace FLECS

#endif // B9FC4679_2559_4145_830A_947F1BB2CF52
