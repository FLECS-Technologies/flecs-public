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

#include "archive/impl/write_base.h"

#include <archive.h>

#include "archive/impl/archive_entry.h"

namespace flecs {

write_base_t::~write_base_t()
{
    close();
}

auto write_base_t::write_header(archive_entry_t& entry) //
    -> int
{
    return archive_write_header(get(), entry.get());
}

auto write_base_t::do_close() //
    -> void
{
    archive_write_close(get());
    archive_write_free(get());
}

} // namespace flecs
