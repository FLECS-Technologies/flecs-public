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

#include "flecs/util/udev/udev.h"

#include <libudev.h>

#include <utility>

namespace flecs {
namespace udev {

udev_t::udev_t()
    : _handle{udev_new()}
    , _owner{std::this_thread::get_id()}
{}

udev_t::udev_t(const udev_t& other)
    : _handle{udev_ref(*other)}
    , _owner{other._owner}
{
    validate_owner();
}

udev_t::udev_t(udev_t&& other)
    : _handle{}
    , _owner{std::this_thread::get_id()}
{
    swap(*this, other);
}

udev_t& udev_t::operator=(udev_t other)
{
    swap(*this, other);
    return *this;
}

udev_t::~udev_t()
{
    if (_handle) {
        validate_owner();
        udev_unref(_handle);
    }
}

auto swap(udev_t& lhs, udev_t& rhs) //
    -> void
{
    lhs.validate_owner();
    rhs.validate_owner();

    using std::swap;
    swap(lhs._handle, rhs._handle);
    swap(lhs._owner, rhs._owner);
}

auto udev_t::validate_owner() //
    -> void
{
    if (_handle && _owner != std::this_thread::get_id()) {
        udev_unref(**this);
        throw std::runtime_error{"Cannot re-use udev handle in different thread"};
    }
}

} // namespace udev
} // namespace flecs
