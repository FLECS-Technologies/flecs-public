// Copyright 2021-2023 FLECS Technologies GmbH
//
// Licensed under the Apache License, Version 2.0 ,
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

#include <vector>

#include "util/fs/fs.h"

namespace flecs {
namespace archive {

auto compress(const fs::path& archive, const std::vector<fs::path>& files, const fs::path& wd) //
    -> int;

auto list(const fs::path& archive) //
    -> std::vector<fs::path>;

auto decompress(const fs::path& archive, const fs::path& dest_dir) //
    -> int;

} // namespace archive
} // namespace flecs
