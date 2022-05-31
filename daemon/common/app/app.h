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

#ifndef E02D9ED5_6E61_4F4F_B9B2_C94F79443A61
#define E02D9ED5_6E61_4F4F_B9B2_C94F79443A61

#include <string>

#include "app_status.h"
#include "manifest/manifest.h"

namespace FLECS {

class app_t
{
public:
    app_t();

    app_t(const std::string& manifest_path, app_status_e status, app_status_e desired);

    auto& app() const noexcept { return _manifest.app(); }
    auto& version() const noexcept { return _manifest.version(); }

private:
    friend void to_json(json_t& j, const app_t& app);

    app_manifest_t _manifest;
    app_status_e _status;
    app_status_e _desired;
};

} // namespace FLECS

#endif // E02D9ED5_6E61_4F4F_B9B2_C94F79443A61
