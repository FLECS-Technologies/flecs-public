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

#include <string>
#include <string_view>

namespace flecs {
namespace apps {

/** @brief String wrapper class that validates App names against the specification.
 *
 * All App names correspond to the following schema:
 *  - reverse domain name notation
 *  - at least three sections, where:
 *      o the first section is the top-level-domain (e.g. tech)
 *      o the second section is the company name (e.g. flecs)
 *      o the third section is the product name (e.g. service-mesh)
 *      o sections are separated by dots (e.g. resulting in tech.flecs.service-mesh)
 *  - allowed characters:
 *      o top-level-domain: [a-z]+
 *      o company name: ([a-z0-9]|[a-z0-9]+[a-z0-9\-]*[a-z0-9]+) : must start and end with [a-z0-9]
 *      o product name: ([a-z0-9]|[a-z0-9]+[a-z0-9\-.]*[a-z0-9]+): must start and end with [a-z0-9]
 *  - maximum length: 127 characters
 *
 */
class name_t
{
public:
    static constexpr auto MAX_APP_NAME_LEN = 127;

    name_t() = default;
    name_t(std::string app_name);

    auto is_valid() const noexcept //
        -> bool;

    auto value() const noexcept //
        -> const std::string&;

private:
    friend auto operator<=>(const name_t&, const name_t&) = default;

    std::string _app_name;
};

} // namespace apps
} // namespace flecs
