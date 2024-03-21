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

#include "flecs/util/archive/impl/write_archive.h"

#include <archive.h>

namespace flecs {

write_archive_t::write_archive_t(const fs::path& archive)
    : write_base_t{archive_write_new()}
{
    if (!*this) {
        return;
    }

    auto ext_primary = archive.extension();
    auto ext_secondary = archive.stem().extension();
    if (ext_primary == ".gz" && ext_secondary == ".tar") {
        archive_write_add_filter_gzip(get());
        archive_write_set_format_gnutar(get());
    } else if (ext_primary == ".zip") {
        archive_write_set_format_zip(get());
    } else if (ext_primary == ".tar") {
        archive_write_set_format_gnutar(get());
    } else {
        std::fprintf(stderr, "Unknown extension %s%s\n", ext_secondary.c_str(), ext_primary.c_str());
        close();
        return;
    }

    auto ec = std::error_code{};
    fs::create_directories(archive.parent_path(), ec);
    if (ec) {
        std::fprintf(
            stderr,
            "Could not create directory %s: %d\n",
            archive.parent_path().c_str(),
            ec.value());
        close();
    }

    auto res = archive_write_open_filename(get(), archive.c_str());
    if (res != ARCHIVE_OK) {
        log_error(res, __LINE__);
        close();
        return;
    }
}

auto write_archive_t::write_data(const void* buf, std::size_t len) //
    -> int
{
    return archive_write_data(get(), buf, len);
}

write_archive_t::~write_archive_t()
{
    close();
}

} // namespace flecs
