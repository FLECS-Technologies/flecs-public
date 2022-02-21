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

#include "libflecs.h"

int main(int argc, char** argv)
{
    auto lib = FLECS::libflecs_t{};
    auto res = lib.run_command(argc, argv);
    std::fprintf((res == 0) ? stdout : stderr, "%s", lib.response().c_str());
    return res;
}
