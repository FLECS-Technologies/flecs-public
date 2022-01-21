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

#ifndef FLECS_util_sqlite3_primary_h
#define FLECS_util_sqlite3_primary_h

#include <string>

namespace FLECS {

struct sqlite3_primary_t
{
public:
    template <typename... Args>
    sqlite3_primary_t(Args&&... args)
        : _primary{"PRIMARY KEY("}
    {
        _primary.append(FLECS::stringify_delim(',', args...));
        _primary.append(")");
    }

    auto value() const noexcept { return _primary; }

private:
    std::string _primary;
};

inline std::string to_string(const sqlite3_primary_t& primary)
{
    return primary.value();
}

} // namespace FLECS

#endif // FLECS_util_sqlite3_primary_h
