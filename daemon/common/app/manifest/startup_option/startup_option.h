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

#ifndef BD070798_4248_4392_A589_BD18FED5003E
#define BD070798_4248_4392_A589_BD18FED5003E

#include <string_view>

namespace FLECS {

enum class startup_option_t : unsigned {
    INVALID = 0x00000000,
    INIT_NETWORK_AFTER_START = 0x00000001,
};

auto startup_option_from_string(std::string_view str) //
    -> startup_option_t;

} // namespace FLECS

#endif // BD070798_4248_4392_A589_BD18FED5003E
