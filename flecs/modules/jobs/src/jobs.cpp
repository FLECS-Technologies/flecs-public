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

#include "flecs/modules/jobs/jobs.h"

#include "flecs/modules/factory/factory.h"
#include "flecs/modules/jobs/impl/jobs_impl.h"

namespace flecs {
namespace module {

jobs_t::jobs_t()
    : _impl{new impl::jobs_t{}}
{}

jobs_t::~jobs_t() = default;

auto jobs_t::do_init() //
    -> void
{
    FLECS_V2_ROUTE("/jobs").methods("GET"_method)([this]() { return list_jobs(); });
    FLECS_V2_ROUTE("/jobs/<uint>").methods("GET"_method)([this](std::uint32_t job_id) {
        return get_job(jobs::id_t{job_id});
    });
    FLECS_V2_ROUTE("/jobs/<uint>").methods("DELETE"_method)([this](std::uint32_t job_id) {
        return delete_job(jobs::id_t{job_id});
    });

    _impl->do_init();
}

auto jobs_t::do_deinit() //
    -> void
{
    _impl->do_deinit();
}

auto jobs_t::append(jobs::job_t job, std::string desc) //
    -> jobs::id_t
{
    return _impl->do_append(std::move(job), std::move(desc));
}

auto jobs_t::list_jobs() const //
    -> crow::response
{
    return _impl->do_list_jobs();
}

auto jobs_t::get_job(jobs::id_t job_id) const //
    -> crow::response
{
    return _impl->do_get_job(std::move(job_id));
}

auto jobs_t::delete_job(jobs::id_t job_id) //
    -> crow::response
{
    return _impl->do_delete_job(std::move(job_id));
}

auto jobs_t::wait_for_job(jobs::id_t job_id) const //
    -> result_t
{
    return _impl->do_wait_for_job(std::move(job_id));
}

} // namespace module
} // namespace flecs
