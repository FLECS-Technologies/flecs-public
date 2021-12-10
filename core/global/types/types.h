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

#ifndef FLECS_core_global_types_h
#define FLECS_core_global_types_h

#include <cstdint>
#include <type_traits>

namespace FLECS {

using void_t = void;
using bool_t = bool;

using int8_t = std::int8_t;
using int16_t = std::int16_t;
using int32_t = std::int32_t;
using int64_t = std::int64_t;

using uint8_t = std::make_unsigned_t<int8_t>;
using uint16_t = std::make_unsigned_t<int16_t>;
using uint32_t = std::make_unsigned_t<int32_t>;
using uint64_t = std::make_unsigned_t<int64_t>;

using fp32_t = float;
using fp64_t = double;

using size_t = std::size_t;
using ssize_t = std::make_signed_t<size_t>;

using char_t = char;

using cstr_t = const char_t*;
using u16cstr_t = const char16_t*;
using u32cstr_t = const char32_t*;
using wcstr_t = const wchar_t*;

} // namespace FLECS

#endif // FLECS_core_global_types_h
