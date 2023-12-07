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

#include "archive/impl/write_disk.h"

#include <archive.h>

namespace flecs {

write_disk_t::write_disk_t(const fs::path& path)
    : write_base_t{archive_write_disk_new()}
{
    if (!*this) {
        return;
    }

    auto flags =
        ARCHIVE_EXTRACT_ACL | ARCHIVE_EXTRACT_FFLAGS | ARCHIVE_EXTRACT_TIME | ARCHIVE_EXTRACT_PERM;

    archive_write_disk_set_options(get(), flags);
    archive_write_disk_set_standard_lookup(get());

    auto ec = std::error_code{};
    fs::create_directories(path, ec);
    if (ec) {
        std::fprintf(stderr, "Could not create directory %s: %d\n", path.c_str(), ec.value());
        close();
    }
}

auto write_disk_t::write_data_block(const void* data, size_t len, std::int64_t offset) //
    -> int
{
    return archive_write_data_block(get(), data, len, offset);
}

write_disk_t::~write_disk_t()
{
    close();
}

} // namespace flecs
