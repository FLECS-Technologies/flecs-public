// Copyright 2021-2023 FLECS Technologies GmbH
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

#pragma once

#include <cstdint>

#include "base_common.h"

struct stat;
struct archive_entry;

namespace FLECS {

class archive_entry_t : public archive::base_t<::archive_entry>
{
public:
    using base_t::base_t;

    archive_entry_t();
    ~archive_entry_t();

    auto set_pathname(const char* path) //
        -> void;
    auto copy_stat(const struct stat* st) //
        -> void;

    auto pathname() //
        -> const char*;
    auto size() //
        -> std::int64_t;

private:
    auto do_close() //
        -> void override;
};

} // namespace FLECS
