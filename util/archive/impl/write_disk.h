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

#include "util/fs/fs.h"
#include "write_base.h"

struct archive;

namespace flecs {

class write_disk_t : public write_base_t
{
public:
    using write_base_t::write_base_t;

    explicit write_disk_t(const fs::path& path);
    ~write_disk_t();

    auto write_data_block(const void* data, size_t len, std::int64_t offset) //
        -> int;
};

} // namespace flecs
