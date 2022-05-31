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

#ifndef B924454A_EE02_4516_BAEE_4F84C9EC14AB
#define B924454A_EE02_4516_BAEE_4F84C9EC14AB

#include <string>

#include "util/json/json.h"

namespace FLECS {

class volume_t
{
public:
    enum volume_type_t
    {
        NONE,
        BIND_MOUNT,
        VOLUME,
    };

    volume_t() noexcept;

    explicit volume_t(const std::string& volume_str) noexcept;

    bool is_valid() const noexcept;

    auto& host() const noexcept { return _host; }
    auto& container() const noexcept { return _container; }
    auto& type() const noexcept { return _type; }

private:
    friend void to_json(json_t& j, const volume_t& volume);

    std::string _host;
    std::string _container;
    volume_type_t _type;
};

inline bool operator<(const volume_t& lhs, const volume_t& rhs)
{
    return lhs.host() < rhs.host();
}

inline bool operator==(const volume_t& lhs, const volume_t& rhs)
{
    return lhs.host() == rhs.host();
}

inline bool operator!=(const volume_t& lhs, const volume_t& rhs)
{
    return !(lhs.host() == rhs.host());
}

inline std::string to_string(const volume_t::volume_type_t& volume_type)
{
    switch (volume_type)
    {
        case volume_t::BIND_MOUNT:
            return "bind mount";
        case volume_t::VOLUME:
            return "volume";
        default:
            return "unknown";
    }
}

} // namespace FLECS

#endif // B924454A_EE02_4516_BAEE_4F84C9EC14AB
