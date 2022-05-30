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

#ifndef D9E7247E_A1EF_469A_9089_3EA80D84D2E4
#define D9E7247E_A1EF_469A_9089_3EA80D84D2E4

#include <string>

#include "util/json/json.h"

namespace FLECS {

class conffile_t
{
public:
    conffile_t()
        : _local{}
        , _container{}
        , _ro{}
        , _init{}
    {}

    explicit conffile_t(const std::string& str);

    auto& local() const noexcept { return _local; }
    void local(std::string local) { _local = local; }

    auto& container() const noexcept { return _container; }
    void container(std::string container) { _container = container; }

    auto ro() const noexcept { return _ro; }
    void ro(bool ro) { _ro = ro; }

    auto init() const noexcept { return _init; }
    void init(bool init) { _init = init; }

    bool is_valid() const noexcept;

private:
    friend void to_json(json_t& j, const conffile_t& conffile);

    std::string _local;
    std::string _container;
    bool _ro;
    bool _init;
};

inline bool operator<(const conffile_t& lhs, const conffile_t& rhs)
{
    return lhs.local() < rhs.local();
}

inline bool operator==(const conffile_t& lhs, const conffile_t& rhs)
{
    return lhs.local() == rhs.local();
}

} // namespace FLECS

#endif // D9E7247E_A1EF_469A_9089_3EA80D84D2E4
