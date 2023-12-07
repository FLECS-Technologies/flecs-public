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

#include "archive/impl/base_archive.h"

#include <archive.h>

namespace flecs {

auto base_archive_t::log_error(int res, int where) //
    -> void
{
    std::fprintf(
        stderr,
        "(libarchive) %s @%d: %d (%s)\n",
        (res == archive::Warn) ? "warning" : "error",
        where,
        error_code(),
        error_string());
}

auto base_archive_t::error_code() //
    -> int
{
    return archive_errno(get());
}

auto base_archive_t::error_string() //
    -> const char*
{
    return archive_error_string(get());
}

} // namespace flecs
