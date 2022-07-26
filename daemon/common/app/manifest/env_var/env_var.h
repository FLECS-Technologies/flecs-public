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

#ifndef FDADEFFE_6E66_4E93_8B9D_DDBA6629DEC1
#define FDADEFFE_6E66_4E93_8B9D_DDBA6629DEC1

#include <string>

#include "util/json/json.h"

namespace FLECS {

class env_var_t
{
public:
    env_var_t()
        : _var{}
    {}

    env_var_t(std::string var)
        : _var{var}
    {}

    bool is_valid() const noexcept;

    auto& var() const noexcept { return _var; }

private:
    std::string _var;
};

inline bool operator<(const env_var_t& lhs, const env_var_t& rhs)
{
    return lhs.var() < rhs.var();
}

inline bool operator==(const env_var_t& lhs, const env_var_t& rhs)
{
    return lhs.var() == rhs.var();
}

class mapped_env_var_t
{
public:
    mapped_env_var_t()
        : _env_var{}
        , _value{}
    {}

    mapped_env_var_t(env_var_t var, std::string value)
        : _env_var{var}
        , _value{value}
    {}

    mapped_env_var_t(const std::string& str);

    bool is_valid() const noexcept { return _env_var.is_valid(); }

    auto& var() const noexcept { return _env_var.var(); }
    auto& value() const noexcept { return _value; }

private:
    friend auto to_json(json_t& json, const mapped_env_var_t& mapped_env_var) //
        -> void;

    friend auto from_json(const json_t& json, mapped_env_var_t& mapped_env_var) //
        -> void;

    env_var_t _env_var;
    std::string _value;
};

inline bool operator<(const mapped_env_var_t& lhs, const mapped_env_var_t& rhs)
{
    return lhs.var() < rhs.var();
}

inline bool operator==(const mapped_env_var_t& lhs, const mapped_env_var_t& rhs)
{
    return lhs.var() == rhs.var();
}

inline bool operator!=(const mapped_env_var_t& lhs, const mapped_env_var_t& rhs)
{
    return !(lhs == rhs);
}

inline std::string to_string(const mapped_env_var_t& mapped_env_var)
{
    auto res = std::string{};
    if (mapped_env_var.is_valid())
    {
        res += mapped_env_var.var() + "=" + mapped_env_var.value();
    }
    return res;
}

} // namespace FLECS

#endif // FDADEFFE_6E66_4E93_8B9D_DDBA6629DEC1
