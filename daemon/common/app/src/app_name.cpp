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

#include "app_name.h"

#include <regex>

namespace flecs {

app_name_t::app_name_t(std::string app_name)
    : _app_name{}
{
    const auto app_regex = std::regex{R"-(^(?:[a-z]+)[.])-"
                                      R"-((?:(?:[a-z0-9]|[a-z0-9]+[a-z0-9\-]*[a-z0-9]+)[.])+)-"
                                      R"-((?:[a-z0-9]|[a-z0-9]+[a-z0-9\-]*[a-z0-9]+)$)-"};

    if ((app_name.length() <= MAX_APP_NAME_LEN) && (std::regex_match(app_name, app_regex))) {
        _app_name = std::move(app_name);
    }
}

auto app_name_t::is_valid() const noexcept //
    -> bool
{
    return !_app_name.empty();
}

auto app_name_t::value() const noexcept //
    -> std::string_view
{
    return _app_name;
}

} // namespace flecs
