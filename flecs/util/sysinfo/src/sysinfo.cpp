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

#include "flecs/util/sysinfo/sysinfo.h"

#include "cxxbridge/flecs_core_cxx_bridge/src/lib.rs.h"
#include "flecs/util/cxx23/string.h"

namespace flecs {

sysinfo_t::sysinfo_t()
    : _kernel_version{}
    , _kernel_build{}
    , _machine{}
    , _distro_id{}
    , _distro_code{}
    , _distro_name{}
    , _distro_version{}
    , _arch{}
    , _platform{}
{
    auto source = read_system_info();

    _kernel_version = std::string(source.kernel.version);
    _kernel_build = std::string(source.kernel.build);
    _machine = std::string(source.kernel.machine);

    _arch = std::string(source.arch);
    _platform = std::string(source.platform);

    _distro_code = std::string(source.distro.codename);
    _distro_id = std::string(source.distro.id);
    _distro_name = std::string(source.distro.name);
    _distro_version = std::string(source.distro.version);
}

auto sysinfo_t::arch() const noexcept //
    -> const std::string&
{
    return _arch;
}

auto to_json(json_t& j, const sysinfo_t& sysinfo) //
    -> void
{
    j = json_t{
        {"arch", sysinfo._arch},
        {"distro",
         {{"codename", sysinfo._distro_code},
          {"id", sysinfo._distro_id},
          {"name", sysinfo._distro_name},
          {"version", sysinfo._distro_version}}},
        {"kernel",
         {{"build", sysinfo._kernel_build},
          {"machine", sysinfo._machine},
          {"version", sysinfo._kernel_version}}},
        {"platform", sysinfo._platform},
    };
}

auto from_json(const json_t& j, sysinfo_t& sysinfo) //
    -> void
{
    try {
        j.at("arch").get_to(sysinfo._arch);
        j.at("distro").at("codename").get_to(sysinfo._distro_code);
        j.at("distro").at("id").get_to(sysinfo._distro_id);
        j.at("distro").at("name").get_to(sysinfo._distro_name);
        j.at("distro").at("version").get_to(sysinfo._distro_version);
        j.at("kernel").at("build").get_to(sysinfo._kernel_build);
        j.at("kernel").at("machine").get_to(sysinfo._machine);
        j.at("kernel").at("version").get_to(sysinfo._kernel_version);
        j.at("platform").get_to(sysinfo._platform);
    } catch (...) {
        sysinfo = {};
    }
}

} // namespace flecs
