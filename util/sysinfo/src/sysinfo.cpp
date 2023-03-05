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

#include "sysinfo.h"

#include <sys/utsname.h>

#include <fstream>
#include <map>
#include <regex>

#include "util/cxx20/string.h"

namespace FLECS {

sysinfo_t::sysinfo_t()
    : _os{}
    , _kernel_version{}
    , _kernel_build{}
    , _machine{}
    , _distro_id{}
    , _distro_code{}
    , _distro_name{}
    , _distro_version{}
    , _arch{}
    , _platform{}
{
    auto buf = utsname{};
    const auto res = uname(&buf);
    if (res < 0) {
        return;
    }

    _os = buf.sysname;
    _kernel_version = buf.release;
    _kernel_build = buf.version;
    _machine = buf.machine;

    auto ec = std::error_code{};
    if (fs::exists("/etc/os-release", ec)) {
        parse_os_release("/etc/os-release");
    } else if (fs::exists("/usr/lib/os-release")) {
        parse_os_release("/usr/lib/os-release");
    }
    if (cxx20::contains(_kernel_version, "weidmueller")) {
        _platform = "weidmueller";
    }
}

auto sysinfo_t::parse_os_release(fs::path path) //
    -> void
{
    const auto codename_regex = std::regex{R"#(^VERSION_CODENAME=(?:"(.+)"|(.+))$)#"};
    const auto id_regex = std::regex{R"#(^ID=(?:"(.+)"|(.+))$)#"};
    const auto name_regex = std::regex{R"#(^PRETTY_NAME=(?:"(.+)"|(.+))$)#"};
    const auto version_regex = std::regex{R"#(^VERSION_ID=(?:"(.+)"|(.+))$)#"};

    auto os_release = std::ifstream{path};

    auto line = std::string{};
    while (std::getline(os_release, line)) {
        auto m = std::smatch{};
        if (std::regex_match(line, m, codename_regex)) {
            _distro_code = m[1].matched ? m[1] : m[2];
        } else if (std::regex_match(line, m, id_regex)) {
            _distro_id = m[1].matched ? m[1] : m[2];
        } else if (std::regex_match(line, m, name_regex)) {
            _distro_name = m[1].matched ? m[1] : m[2];
        } else if (std::regex_match(line, m, version_regex)) {
            _distro_version = m[1].matched ? m[1] : m[2];
        }
    }
}

auto to_json(json_t& j, const sysinfo_t& sysinfo) //
    -> void
{
    j = json_t{
        {"arch", machine_to_arch(sysinfo._machine)},
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

auto machine_to_arch(std::string_view machine) //
    -> std::string
{
    using std::operator""s;

    const auto m = std::map<std::string_view, std::string_view>{
        {"aarch64", "arm64"},
        {"armv7l", "armhf"},
        {"x86", "i386"},
        {"x86_64", "amd64"}};

    const auto it = m.find(machine);
    return (it != m.cend()) ? std::string{it->second} : "unknown"s;
}

} // namespace FLECS
