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

#include "flecs/util/archive/impl/read_archive.h"

#include <archive.h>

#include "flecs/util/archive/impl/archive_entry.h"

namespace flecs {

read_archive_t::read_archive_t()
    : base_archive_t{archive_read_new()}
{}

read_archive_t::read_archive_t(const fs::path& archive)
    : read_archive_t{}
{
    archive_read_support_format_gnutar(get());
    archive_read_support_format_zip(get());
    archive_read_support_filter_gzip(get());
    if (archive_read_open_filename(get(), archive.c_str(), 0) != ARCHIVE_OK) {
        return;
    }
}

read_archive_t::~read_archive_t() //
{
    close();
}

auto read_archive_t::read_next_header(archive_entry_t& entry) //
    -> int
{
    return archive_read_next_header2(get(), entry.get());
}

auto read_archive_t::read_data_block(const void*& buf, std::size_t& len, std::int64_t& offset) //
    -> int
{
    return archive_read_data_block(get(), &buf, &len, &offset);
}

auto read_archive_t::do_close() //
    -> void
{
    archive_read_close(get());
    archive_read_free(get());
}

} // namespace flecs
