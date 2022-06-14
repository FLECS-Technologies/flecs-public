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

#ifndef C32351A7_25BD_434A_AEDF_7031E4878C37
#define C32351A7_25BD_434A_AEDF_7031E4878C37

#include <filesystem>
#include <set>
#include <string>

#include "conffile/conffile.h"
#include "env_var/env_var.h"
#include "network/network.h"
#include "port_range/port_range.h"
#include "startup_option/startup_option.h"
#include "util/json/json.h"
#include "util/yaml/yaml.h"
#include "volume/volume.h"

namespace FLECS {

class app_manifest_t
{
public:
    using args_t = std::vector<std::string>;
    using conffiles_t = std::vector<conffile_t>;
    using devices_t = std::set<std::string>;
    using envs_t = std::set<mapped_env_var_t>;
    using networks_t = std::vector<network_t>;
    using startup_options_t = std::vector<startup_option_t>;
    using ports_t = std::vector<mapped_port_range_t>;
    using volumes_t = std::vector<volume_t>;

    app_manifest_t();

    static app_manifest_t from_yaml_string(const std::string& yaml);
    static app_manifest_t from_yaml_file(const std::filesystem::path& path);

    auto& yaml_loaded() const noexcept { return _yaml_loaded; }
    auto& yaml_valid() const noexcept { return _yaml_valid; }

    auto& app() const noexcept { return _app; }
    auto& args() const noexcept { return _args; }
    auto& author() const noexcept { return _author; }
    auto& avatar() const noexcept { return _avatar; }
    auto& category() const noexcept { return _category; }
    auto& conffiles() const noexcept { return _conffiles; }
    auto& description() const noexcept { return _description; }
    auto& devices() const noexcept { return _devices; }
    auto& editor() const noexcept { return _editor; }
    auto& env() const noexcept { return _env; }
    auto& hostname() const noexcept { return _hostname; }
    auto& image() const noexcept { return _image; }
    auto image_with_tag() const { return _image + ":" + _version; }
    auto interactive() const noexcept { return _interactive; }
    auto multi_instance() const noexcept { return _multi_instance; }
    auto& networks() const noexcept { return _networks; }
    auto& ports() const noexcept { return _ports; }
    auto& startup_options() const noexcept { return _startup_options; }
    auto& title() const noexcept { return _title; }
    auto& version() const noexcept { return _version; }
    auto& volumes() const noexcept { return _volumes; }

private:
    friend void to_json(json_t& json, const app_manifest_t& app_manifest);

    void parse_yaml(const yaml_t& yaml);
    void validate_yaml();

    bool _yaml_loaded;
    bool _yaml_valid;

    std::string _app;
    args_t _args;
    std::string _author;
    std::string _avatar;
    std::string _category;
    conffiles_t _conffiles;
    std::string _description;
    devices_t _devices;
    std::string _editor;
    envs_t _env;
    std::string _hostname;
    std::string _image;
    bool _interactive;
    bool _multi_instance;
    networks_t _networks;
    ports_t _ports;
    startup_options_t _startup_options;
    std::string _title;
    std::string _version;
    volumes_t _volumes;
};

} // namespace FLECS

#endif // C32351A7_25BD_434A_AEDF_7031E4878C37
