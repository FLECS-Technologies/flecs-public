// Copyright 2021 FLECS Technologies GmbH
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

#ifndef FLECS_util_literals_h
#define FLECS_util_literals_h

namespace FLECS {

constexpr auto operator ""_B(const unsigned long long num)
{
    return num;
}

constexpr auto operator ""_kiB(const unsigned long long num)
{
    return 1024 * operator ""_B(num);
}

constexpr auto operator ""_MiB(const unsigned long long num)
{
    return 1024 * operator ""_kiB(num);
}

constexpr auto operator ""_GiB(const unsigned long long num)
{
    return 1024 * operator ""_MiB(num);
}

constexpr auto operator ""_TiB(const unsigned long long num)
{
    return 1024 * operator ""_GiB(num);
}

} //namespace FLECS

#endif //FLECS_util_literals_h
