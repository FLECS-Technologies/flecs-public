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

#include "core/global/types/types.h"

namespace FLECS {

// static_assert size
static_assert(sizeof(bool_t) == 1);

static_assert(sizeof(int8_t) == 1);
static_assert(sizeof(int16_t) == 2);
static_assert(sizeof(int32_t) == 4);
static_assert(sizeof(int64_t) == 8);

static_assert(sizeof(uint8_t) == 1);
static_assert(sizeof(uint16_t) == 2);
static_assert(sizeof(uint32_t) == 4);
static_assert(sizeof(uint64_t) == 8);

static_assert(sizeof(fp32_t) == 4);
static_assert(sizeof(fp64_t) == 8);

static_assert(sizeof(char_t) == 1);
static_assert(sizeof(char16_t) == 2);
static_assert(sizeof(char32_t) == 4);;

static_assert(std::is_same_v<decltype(sizeof(size_t)), size_t>);

// static_assert signedness
static_assert(std::is_unsigned_v<bool_t>);

static_assert(std::is_signed_v<int8_t>);
static_assert(std::is_signed_v<int16_t>);
static_assert(std::is_signed_v<int32_t>);
static_assert(std::is_signed_v<int64_t>);

static_assert(std::is_unsigned_v<uint8_t>);
static_assert(std::is_unsigned_v<uint16_t>);
static_assert(std::is_unsigned_v<uint32_t>);
static_assert(std::is_unsigned_v<uint64_t>);

static_assert(std::is_signed_v<fp32_t>);
static_assert(std::is_signed_v<fp64_t>);

static_assert(std::is_unsigned_v<size_t>);
static_assert(std::is_signed_v<ssize_t>);

// static_assert floatiness
static_assert(std::is_floating_point_v<fp32_t>);
static_assert(std::is_floating_point_v<fp64_t>);

// static_assert pointiness
static_assert(sizeof(cstr_t) == sizeof(void*));
static_assert(sizeof(u16cstr_t) == sizeof(void*));
static_assert(sizeof(u32cstr_t) == sizeof(void*));
static_assert(sizeof(wcstr_t) == sizeof(void*));

} // namespace FLECS
