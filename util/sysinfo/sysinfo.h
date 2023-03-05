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

#pragma once

#include <string>

#include "util/fs/fs.h"
#include "util/json/json.h"

namespace FLECS {

class sysinfo_t
{
public:
    sysinfo_t();

    auto arch() const noexcept //
        -> const std::string&;

private:
    friend auto to_json(json_t& j, const sysinfo_t& sysinfo) //
        -> void;

    friend auto from_json(const json_t& j, sysinfo_t& sysinfo) //
        -> void;

    auto parse_os_release(fs::path path) //
        -> void;

    // uname info
    std::string _os;             // sysname
    std::string _kernel_version; // release
    std::string _kernel_build;   // version
    std::string _machine;        // machine

    // os-release info
    std::string _distro_id;      // e.g. debian
    std::string _distro_code;    // e.g. bullseye
    std::string _distro_name;    // e.g. Debian GNU/Linux 11 (bullseye)
    std::string _distro_version; // e.g. 11

    // additional info t.b.d.
    std::string _arch;
    std::string _platform;
};

auto machine_to_arch(std::string_view machine) //
    -> std::string;

} // namespace FLECS
