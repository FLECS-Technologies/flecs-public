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

#include "daemon/api/lib/libflecs.h"

#include <map>
#include <string>

#include "daemon/api/lib/impl/libflecs_impl.h"

namespace flecs {

template <typename Impl>
libflecs_t<Impl>::libflecs_t()
    : _impl{new Impl{}}
{}

template <typename Impl>
libflecs_t<Impl>::~libflecs_t()
{
    disconnect();
}

template <typename Impl>
int libflecs_t<Impl>::connect(const std::string& host, int port)
{
    return _impl->do_connect(host, port);
}

template <typename Impl>
int libflecs_t<Impl>::disconnect()
{
    return _impl->do_disconnect();
}

// app management
template <typename Impl>
int libflecs_t<Impl>::install_app(
    const std::string& app, const std::string& version, const std::string& license)
{
    return _impl->do_install_app(app, version, license);
}

template <typename Impl>
int libflecs_t<Impl>::uninstall_app(const std::string& app, const std::string& version)
{
    return _impl->do_uninstall_app(app, version);
}

template <typename Impl>
int libflecs_t<Impl>::sideload_app_from_yaml(const std::string& yaml)
{
    return _impl->do_sideload_app_from_yaml(yaml);
}

template <typename Impl>
int libflecs_t<Impl>::sideload_app_from_file(const fs::path& manifest_path)
{
    return _impl->do_sideload_app_from_file(manifest_path);
}

template <typename Impl>
int libflecs_t<Impl>::list_apps()
{
    return _impl->do_list_apps();
}

template <typename Impl>
int libflecs_t<Impl>::list_instances(const std::string& app, const std::string& version)
{
    return _impl->do_list_instances(app, version);
}

template <typename Impl>
int libflecs_t<Impl>::list_versions(const std::string& app)
{
    return _impl->do_list_versions(app);
}

// instance management
template <typename Impl>
int libflecs_t<Impl>::create_instance(
    const std::string& app, const std::string& version, const std::string& instanceName)
{
    return _impl->do_create_instance(app, version, instanceName);
}

template <typename Impl>
int libflecs_t<Impl>::delete_instance(
    const std::string& instanceId, const std::string& app, const std::string& version)
{
    return _impl->do_delete_instance(instanceId, app, version);
}

template <typename Impl>
int libflecs_t<Impl>::start_instance(
    const std::string& instanceId, const std::string& app, const std::string& version)
{
    return _impl->do_start_instance(instanceId, app, version);
}

template <typename Impl>
int libflecs_t<Impl>::stop_instance(
    const std::string& instanceId, const std::string& app, const std::string& version)
{
    return _impl->do_stop_instance(instanceId, app, version);
}

// system info
template <typename Impl>
int libflecs_t<Impl>::version()
{
    return _impl->do_version();
}

template <typename Impl>
int libflecs_t<Impl>::ping()
{
    return _impl->do_ping();
}

// string-based interface
template <typename Impl>
int libflecs_t<Impl>::run_command(const std::string& command, const std::vector<std::string>& args)
{
    return _impl->do_run_command(command, args);
}

template <typename Impl>
int libflecs_t<Impl>::run_command(int argc, char** argv)
{
    /* argv[0] represents program name - skip */
    auto command = argc > 1 ? argv[1] : "";
    auto args = std::vector<std::string>{};
    for (auto i = 2; i < argc; ++i) {
        args.push_back(argv[i]);
    }

    return run_command(command, args);
}

template <typename Impl>
int libflecs_t<Impl>::response_code() const noexcept
{
    return _impl->do_response_code();
}

// retrieve response as formatted JSON string
template <typename Impl>
std::string libflecs_t<Impl>::json_response() const noexcept
{
    return _impl->do_json_response();
}

template class libflecs_t<impl::libflecs_t>;

} // namespace flecs
