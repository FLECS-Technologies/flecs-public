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

#include "archive/archive.h"

#include <errno.h>
#include <sys/stat.h>

#include <algorithm>
#include <cstring>
#include <fstream>
#include <memory>

#include "archive/impl/archive_entry.h"
#include "archive/impl/read_archive.h"
#include "archive/impl/write_archive.h"
#include "archive/impl/write_disk.h"
#include "util/cxx20/string.h"
#include "util/string/literals.h"

namespace FLECS {
namespace archive {

static auto compress_file(write_archive_t& aw, const fs::path& file, const fs::path& wd) //
    -> int
{
    /* write archive header */
    {
        /* determine relative path */
        auto ec = std::error_code{};
        auto rel_path = fs::relative(file.lexically_normal(), wd.lexically_normal(), ec);
        if (ec) {
            std::fprintf(
                stderr,
                "Could not build path relative to %s for %s\n",
                wd.c_str(),
                file.c_str());
            return -1;
        }
        while (!rel_path.empty() && (*rel_path.begin() == "..")) {
            rel_path = rel_path.lexically_relative("../");
        }

        /* stat input file */
        auto st = (struct stat){};
        if (stat(file.c_str(), &st) != 0) {
            std::fprintf(stderr, "Could not stat() %s\n", file.c_str());
            return -1;
        }

        auto entry = archive_entry_t{};
        if (!entry) {
            std::fprintf(stderr, "Could not create archive entry\n");
            return -1;
        }
        entry.set_pathname(rel_path.c_str());
        entry.copy_stat(&st);
        if (auto res = aw.write_header(entry); res != Ok) {
            aw.log_error(res, __LINE__);
            if (res < Warn) {
                return -1;
            }
        }
    }

    /* write payload */
    {
        /* open input file */
        auto f = std::ifstream{file.c_str(), std::ios_base::in | std::ios_base::binary};
        if (!f) {
            std::fprintf(stderr, "Could not open input file %s\n", file.c_str());
            return -1;
        }

        auto buf = std::unique_ptr<char[]>{new char[1_MiB]};
        f.read(buf.get(), 1_MiB);
        while (f.gcount()) {
            auto written = aw.write_data(buf.get(), f.gcount());
            if (written != f.gcount()) {
                aw.log_error(Fatal, __LINE__);
                return -1;
            }
            f.read(buf.get(), 1_MiB);
        }
    }

    return 0;
}

static auto compress_dir(write_archive_t& aw, const fs::path& file, const fs::path& wd) //
    -> int
{
    auto ec = std::error_code{};
    for (auto it = fs::recursive_directory_iterator(file, ec);
         it != fs::recursive_directory_iterator{};
         ++it) {
        auto status = fs::status(*it, ec);
        if (status.type() == fs::file_type::directory) {
            continue;
        } else {
            if (compress_file(aw, *it, wd) != 0) {
                return -1;
            };
        }
    }

    return 0;
}

auto compress(const fs::path& archive, const std::vector<fs::path>& files, const fs::path& wd) //
    -> int
{
    auto ec = std::error_code{};
    if (!fs::is_directory(wd)) {
        return -1;
    }

    auto aw = write_archive_t{archive};
    if (!aw) {
        return -1;
    }

    for (const auto& file : files) {
        auto status = fs::status(file, ec);
        if (status.type() == fs::file_type::directory) {
            if (compress_dir(aw, file, wd) != 0) {
                return -1;
            }
        } else {
            if (compress_file(aw, file, wd) != 0) {
                return -1;
            }
        }
    }

    return 0;
}

auto list(const fs::path& archive) //
    -> std::vector<fs::path>
{
    auto ar = read_archive_t{archive};
    if (!ar) {
        return {};
    }

    auto ret = std::vector<fs::path>{};

    auto entry = archive_entry_t{};
    while (ar.read_next_header(entry) == 0) {
        ret.emplace_back(entry.pathname());
    }

    return ret;
}

auto decompress(const fs::path& archive, const fs::path& dest_dir) //
    -> int
{
    auto ar = read_archive_t{archive};
    if (!ar) {
        return -1;
    }

    auto aw = write_disk_t{dest_dir};
    if (!aw) {
        return -1;
    }

    auto entry = archive_entry_t{};
    while (true) {
        if (auto res = ar.read_next_header(entry); res != Ok) {
            if (res == EndOfFile) {
                break;
            } else {
                ar.log_error(res, __LINE__);
                if (res < Warn) {
                    return -1;
                }
            }
        }
        entry.set_pathname((dest_dir / entry.pathname()).c_str());
        if (auto res = aw.write_header(entry); res != Ok) {
            ar.log_error(res, __LINE__);
            if (res < Warn) {
                return -1;
            }
        }

        if (entry.size() > 0) {
            auto size = std::size_t{};
            auto offset = std::int64_t{};
            auto buf = static_cast<const void*>(nullptr);
            while (true) {
                if (auto res = ar.read_data_block(buf, size, offset); res != Ok) {
                    if (res == EndOfFile) {
                        break;
                    } else {
                        ar.log_error(res, __LINE__);
                        if (res < Warn) {
                            return -1;
                        }
                    }
                }
                if (auto res = aw.write_data_block(buf, size, offset); res != Ok) {
                    ar.log_error(res, __LINE__);
                    if (res < Warn) {
                        return -1;
                    }
                }
            }
        }
    }

    return 0;
}

} // namespace archive
} // namespace FLECS
