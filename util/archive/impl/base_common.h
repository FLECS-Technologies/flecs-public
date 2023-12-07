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

#include <cstdio>

namespace flecs {
namespace archive {

enum error_e : int {
    Ok = 0,
    EndOfFile = 1,
    Retry = -10,
    Warn = -20,
    Failed = -25,
    Fatal = -30,
};

template <typename Base>
class base_t
{
public:
    explicit base_t(Base* h)
        : _h{h}
    {}
    base_t(const base_t&) = delete;
    auto operator=(const base_t&) = delete;

    auto get() //
        -> Base*
    {
        return _h;
    }

    auto operator*() //
        -> Base*
    {
        return get();
    }

    operator bool() //
    {
        return _h != nullptr;
    }

protected:
    base_t(base_t&&) = default;
    ~base_t() = default;

    auto close() //
        -> void
    {
        if (*this) {
            do_close();
            _h = static_cast<Base*>(nullptr);
        }
    }

private:
    virtual auto do_close() //
        -> void = 0;

    Base* _h;
};

} // namespace archive
} // namespace flecs
