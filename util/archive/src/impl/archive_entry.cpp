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

#include "util/archive/impl/archive_entry.h"

#include <archive_entry.h>

namespace flecs {

archive_entry_t::archive_entry_t()
    : base_t{archive_entry_new()}
{}

archive_entry_t::~archive_entry_t()
{
    close();
}

auto archive_entry_t::set_pathname(const char* path) //
    -> void
{
    archive_entry_set_pathname(get(), path);
}

auto archive_entry_t::copy_stat(const struct stat* st) //
    -> void
{
    archive_entry_copy_stat(get(), st);
}

auto archive_entry_t::pathname() //
    -> const char*
{
    return archive_entry_pathname(get());
}

auto archive_entry_t::size() //
    -> std::int64_t
{
    return archive_entry_size(get());
}

auto archive_entry_t::do_close() //
    -> void
{
    archive_entry_clear(get());
    archive_entry_free(get());
}

} // namespace flecs
