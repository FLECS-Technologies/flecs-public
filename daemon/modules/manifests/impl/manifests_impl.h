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

namespace FLECS {

class module_manifests_t;

namespace impl {

class module_manifests_t
{
    friend class FLECS::module_manifests_t;

public:
    ~module_manifests_t();

private:
    explicit module_manifests_t(FLECS::module_manifests_t* parent);

    auto do_init() //
        -> void;

    auto do_deinit() //
        -> void;

    FLECS::module_manifests_t* _parent;
};

} // namespace impl
} // namespace FLECS
