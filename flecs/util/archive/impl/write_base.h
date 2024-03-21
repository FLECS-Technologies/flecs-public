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

#include "base_archive.h"
#include "flecs/util/fs/fs.h"

struct archive;

namespace flecs {

class archive_entry_t;

class write_base_t : public base_archive_t
{
public:
    using base_archive_t::base_archive_t;

    ~write_base_t();

    auto write_header(archive_entry_t& entry) //
        -> int;

private:
    auto do_close() //
        -> void override;
};

} // namespace flecs
