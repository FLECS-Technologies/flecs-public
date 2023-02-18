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

#include <stdexcept>
#include <thread>

struct udev;

namespace FLECS {
namespace udev {

class udev_t
{
public:
    using value_type = ::udev*;

    udev_t();

    udev_t& operator=(udev_t other);

    udev_t(const udev_t& other);

    udev_t(udev_t&& other);

    ~udev_t();

    friend auto swap(udev_t& lhs, udev_t& rhs) //
        -> void;

    auto operator*() noexcept //
        -> value_type&
    {
        return _handle;
    }

    auto operator*() const noexcept //
        -> const value_type&
    {
        return _handle;
    }

private:
    auto validate_owner() //
        -> void;

    value_type _handle;
    std::thread::id _owner;
};

} // namespace udev
} // namespace FLECS
