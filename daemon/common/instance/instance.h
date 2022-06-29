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

#ifndef BFBA07F5_D230_427A_814B_DA012423C246
#define BFBA07F5_D230_427A_814B_DA012423C246

#include <memory>
#include <string>

#include "instance_config.h"
#include "instance_status.h"

namespace FLECS {

class instance_t
{
public:
    instance_t(
        std::string app,
        std::string version,
        std::string instance_name,
        instance_status_e status,
        instance_status_e desired);

    instance_t(
        std::string id,
        std::string app,
        std::string version,
        std::string instance_name,
        instance_status_e status,
        instance_status_e desired);

    auto id() const noexcept //
        -> const std::string&;
    auto app() const noexcept //
        -> const std::string&;
    auto version() const noexcept //
        -> const std::string&;
    auto instance_name() const noexcept //
        -> const std::string&;
    auto status() const noexcept //
        -> instance_status_e;
    auto desired() const noexcept //
        -> instance_status_e;
    auto config() const noexcept //
        -> const instance_config_t&;
    auto config() noexcept //
        -> instance_config_t&;

    auto regenerate_id() //
        -> void;
    auto instance_name(std::string instance_name) //
        -> void;
    auto status(instance_status_e instance_status) //
        -> void;
    auto desired(instance_status_e instance_status) //
        -> void;

private:
    std::string _id;
    std::string _app;
    std::string _version;
    std::string _instance_name;
    instance_status_e _status;
    instance_status_e _desired;
    instance_config_t _config;
};

inline auto operator==(const instance_t& lhs, const instance_t& rhs) //
    -> bool
{
    return (lhs.id() == rhs.id());
}

inline auto instance_t::id() const noexcept //
    -> const std::string&
{
    return _id;
}

inline auto instance_t::app() const noexcept //
    -> const std::string&
{
    return _app;
}
inline auto instance_t::version() const noexcept //
    -> const std::string&
{
    return _version;
}

inline auto instance_t::instance_name() const noexcept //
    -> const std::string&
{
    return _instance_name;
}

inline auto instance_t::status() const noexcept //
    -> instance_status_e
{
    return _status;
}

inline auto instance_t::desired() const noexcept //
    -> instance_status_e
{
    return _desired;
}

inline auto instance_t::config() noexcept //
    -> instance_config_t&
{
    return _config;
}

inline auto instance_t::config() const noexcept //
    -> const instance_config_t&
{
    return _config;
}

inline auto instance_t::instance_name(std::string instance_name) //
    -> void
{
    _instance_name = instance_name;
}
inline auto instance_t::status(instance_status_e status) //
    -> void
{
    _status = status;
}

inline auto instance_t::desired(instance_status_e desired) //
    -> void
{
    _desired = desired;
}

} // namespace FLECS

#endif // BFBA07F5_D230_427A_814B_DA012423C246
