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

#ifndef C8B89989_19D1_40AE_B788_DD80AE214500
#define C8B89989_19D1_40AE_B788_DD80AE214500

#include <algorithm>
#include <map>
#include <vector>

#include "app_status.h"
#include "env_var/env_var.h"
#include "port_range/port_range.h"
#include "util/cxx20/string.h"
#include "util/string/string_utils.h"

namespace FLECS {

class app_t
{
public:
    using volumes_t = std::map<std::string, std::string>;
    using networks_t = std::vector<std::string>;
    using ports_t = std::vector<mapped_port_range_t>;
    using envs_t = std::vector<mapped_env_var_t>;
    using args_t = std::vector<std::string>;

    app_t() noexcept = default;
    explicit app_t(const std::string& manifest);

    auto yaml_loaded() const noexcept { return _yaml_loaded; }

    auto& name() const noexcept { return _name; }
    void name(std::string name) { _name = name; }

    auto& version() const noexcept { return _version; }
    void version(std::string version) { _version = version; }

    auto& description() const noexcept { return _description; }
    void description(std::string description) { _description = description; }

    auto& author() const noexcept { return _author; }
    void author(std::string author) { _author = author; }

    auto& category() const noexcept { return _category; }
    void category(std::string category) { _category = category; }

    auto& image() const noexcept { return _image; }
    void image(std::string image) { _image = image; }
    auto image_with_tag() const { return _image + ":" + _version; }

    auto& env() const noexcept { return _env; }
    void add_env(envs_t::value_type env) { _env.emplace_back(env); }

    auto& volumes() const noexcept { return _volumes; }
    auto add_volume(volumes_t::key_type local, volumes_t::mapped_type container)
    {
        return _volumes.try_emplace(local, container);
    }
    auto remove_volume(const volumes_t::key_type& local) { return _volumes.erase(local); }

    auto& bind_mounts() const noexcept { return _bind_mounts; }
    auto add_bind_mount(volumes_t::key_type local, volumes_t::mapped_type container)
    {
        return _bind_mounts.try_emplace(local, container);
    }
    auto remove_bind_mount(const volumes_t::key_type& local) { return _bind_mounts.erase(local); }

    auto& networks() const noexcept { return _networks; }
    auto add_network(networks_t::value_type network) { return _networks.emplace_back(network); }
    auto remove_network(const networks_t::value_type& network)
    {
        return _networks.erase(std::remove(_networks.begin(), _networks.end(), network), _networks.end());
    }

    auto& ports() const noexcept { return _ports; }
    void add_port(ports_t::value_type range) { _ports.push_back(range); }

    auto& args() const noexcept { return _args; }
    void add_arg(args_t::value_type arg) { _args.emplace_back(arg); }

    auto interactive() const noexcept { return _interactive; }
    auto interactive(bool interactive) { _interactive = interactive; }

    auto installed_size() const noexcept { return _installed_size; }

    auto multi_instance() const noexcept { return _multi_instance; }
    void multi_instance(bool multi_instance) noexcept { _multi_instance = multi_instance; }

    auto status() const noexcept { return _status; }
    void status(app_status_e status) noexcept { _status = status; }

    auto desired() const noexcept { return _desired; }
    void desired(app_status_e desired) noexcept { _desired = desired; }

private:
    bool _yaml_loaded;

    std::string _name;
    std::string _title;
    std::string _version;
    std::string _description;
    std::string _author;
    std::string _category;
    std::string _image;
    envs_t _env;
    volumes_t _volumes;
    volumes_t _bind_mounts;
    networks_t _networks;
    ports_t _ports;
    args_t _args;
    bool _interactive;
    std::int32_t _installed_size;
    bool _multi_instance;
    app_status_e _status;
    app_status_e _desired;
};

} // namespace FLECS

#endif // C8B89989_19D1_40AE_B788_DD80AE214500
