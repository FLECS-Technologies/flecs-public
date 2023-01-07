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

#ifndef C794C40B_C58D_43F0_8EDC_97D509E67767
#define C794C40B_C58D_43F0_8EDC_97D509E67767

#include <nlohmann/json.hpp>

namespace FLECS {

using json_t = nlohmann::ordered_json;

template <typename InputType>
auto parse_json(InputType&& i)
{
    return nlohmann::ordered_json::parse(std::forward<InputType>(i), nullptr, false, false);
}

inline auto is_valid_json(const json_t& json)
{
    return !json.is_discarded();
}

} // namespace FLECS

#endif // C794C40B_C58D_43F0_8EDC_97D509E67767
