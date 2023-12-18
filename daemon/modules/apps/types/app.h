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

#pragma once

#include <memory>
#include <string>
#include <tuple>

#include "app_key.h"
#include "app_status.h"
#include "util/json/json.h"

namespace flecs {

class app_manifest_t;

namespace apps {

class app_t : key_t
{
public:
    app_t();
    explicit app_t(key_t app_key);
    app_t(key_t app_key, std::shared_ptr<app_manifest_t> manifest);

    auto key() const noexcept //
        -> const key_t&;
    auto installed_size() const noexcept //
        -> std::int64_t;
    auto license_key() const noexcept //
        -> const std::string&;
    auto status() const noexcept //
        -> status_e;
    auto desired() const noexcept //
        -> status_e;
    auto manifest() const noexcept //
        -> std::shared_ptr<app_manifest_t>;

    auto installed_size(std::int64_t installed_size) //
        -> void;
    auto license_key(std::string license_key) //
        -> void;
    auto status(status_e status) //
        -> void;
    auto desired(status_e desired) //
        -> void;
    auto manifest(std::shared_ptr<app_manifest_t> manifest) //
        -> void;

private:
    friend auto to_json(json_t& json, const app_t& app) //
        -> void;
    friend auto from_json(const json_t& json, app_t& app) //
        -> void;

    std::string _license_key;
    std::string _download_token;
    std::int64_t _installed_size;
    status_e _status;
    status_e _desired;
    std::weak_ptr<app_manifest_t> _manifest;
};

} // namespace apps
} // namespace flecs
