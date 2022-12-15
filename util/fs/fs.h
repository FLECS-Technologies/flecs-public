// Copyright 2021-2022 FLECS Technologies GmbH
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

#ifndef A8A6C721_CB1E_4121_A208_E53BB12A2107
#define A8A6C721_CB1E_4121_A208_E53BB12A2107

#include <filesystem>

#include "util/cxx20/string.h"

namespace FLECS {

namespace fs = std::filesystem;

class tmpdir_t
{
public:
    explicit tmpdir_t(fs::path dir) noexcept
        : _dir{std::move(dir)}
    {
        if (!_dir.is_absolute() || !cxx20::starts_with(_dir.c_str(), "/var/lib/flecs/")) {
            _dir.clear();
            return;
        }

        auto ec = std::error_code{};
        fs::create_directories(_dir, ec);
        if (ec) {
            _dir.clear();
        }
    }

    auto created() const noexcept //
        -> bool
    {
        return !_dir.empty();
    }

    ~tmpdir_t()
    {
        if (created()) {
            auto ec = std::error_code{};
            fs::remove_all(_dir, ec);
        }
    }

private:
    fs::path _dir;
};

} // namespace FLECS

#endif /* A8A6C721_CB1E_4121_A208_E53BB12A2107 */
