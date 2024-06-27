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

#include <set>
#include <string>

#include "flecs/common/app/manifest/conffile/conffile.h"
#include "flecs/common/app/manifest/variable/variable.h"
#include "flecs/common/app/manifest/port_range/port_range.h"
#include "flecs/common/app/manifest/startup_option/startup_option.h"
#include "flecs/common/network/network.h"
#include "flecs/util/fs/fs.h"
#include "flecs/util/json/json.h"
#include "flecs/util/yaml/yaml.h"
#include "volume/volume.h"

namespace flecs {

class app_manifest_t
{
public:
    using args_t = std::vector<std::string>;
    using capabilities_t = std::vector<std::string>;
    using conffiles_t = std::vector<conffile_t>;
    using devices_t = std::set<std::string>;
    using envs_t = std::set<mapped_env_var_t>;
    using networks_t = std::vector<network_t>;
    using startup_options_t = std::vector<startup_option_t>;
    using ports_t = std::vector<mapped_port_range_t>;
    using volumes_t = std::vector<volume_t>;
    using labels_t = std::set<mapped_label_var_t>;

    app_manifest_t();

    static app_manifest_t from_json(const json_t& json);
    static app_manifest_t from_yaml(const yaml_t& yaml);

    static app_manifest_t from_json_string(std::string_view string);
    static app_manifest_t from_yaml_string(std::string_view string);

    static app_manifest_t from_json_file(const fs::path& path);
    static app_manifest_t from_yaml_file(const fs::path& path);

    auto& is_valid() const noexcept { return _valid; }

    auto& app() const noexcept { return _app; }
    auto& args() const noexcept { return _args; }
    auto& capabilities() const noexcept { return _capabilities; }
    auto& conffiles() const noexcept { return _conffiles; }
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
    auto& version() const noexcept { return _version; }
    auto& volumes() noexcept { return _volumes; }
    auto& volumes() const noexcept { return _volumes; }
    auto& labels() noexcept { return _labels; }
    auto& labels() const noexcept { return _labels; }

private:
    friend auto to_json(json_t& json, const app_manifest_t& app_manifest) //
        -> void;
    friend auto from_json(const json_t& json, app_manifest_t& app_manifest) //
        -> void;

    void parse_yaml(const yaml_t& yaml);
    void validate();
    void upgrade_manifest_version();

    bool _valid;

    std::string _app;
    std::string _manifest_version;
    args_t _args;
    capabilities_t _capabilities;
    conffiles_t _conffiles;
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
    std::string _version;
    volumes_t _volumes;
    labels_t _labels;
};

} // namespace flecs
