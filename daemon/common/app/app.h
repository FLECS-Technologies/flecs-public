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
#include <tuple>

#include "app_key.h"
#include "app_status.h"
#include "manifest/manifest.h"

namespace FLECS {

class app_t : public app_manifest_t
{
public:
    app_t();

    app_t(const fs::path& manifest_path, app_status_e status, app_status_e desired);
    app_t(const std::string& manifest_string, app_status_e status, app_status_e desired);

    auto& download_token() const noexcept { return _download_token; }
    auto installed_size() const noexcept { return _installed_size; }
    auto& license_key() const noexcept { return _license_key; }
    auto status() const noexcept { return _status; }
    auto desired() const noexcept { return _desired; }

    void download_token(std::string download_token) { _download_token = download_token; }
    void installed_size(std::int32_t installed_size) { _installed_size = installed_size; }
    void license_key(std::string license_key) { _license_key = license_key; }
    void status(app_status_e status) { _status = status; }
    void desired(app_status_e desired) { _desired = desired; }

private:
    friend auto to_json(json_t& json, const app_t& app) //
        -> void;
    friend auto from_json(const json_t& json, app_t& app) //
        -> void;

    app_key_t _key;
    std::string _license_key;
    std::string _download_token;
    std::int32_t _installed_size;
    app_status_e _status;
    app_status_e _desired;
};

} // namespace FLECS

#endif // E02D9ED5_6E61_4F4F_B9B2_C94F79443A61
