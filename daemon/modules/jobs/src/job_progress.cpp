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

#include "job_progress.h"

namespace flecs {

job_progress_t::job_progress_t(job_id_t job_id, std::string desc)
    : _job_id{job_id}
    , _status{}
    , _desc{std::move(desc)}
    , _num_steps{}
    , _current_step{}
    , _result{}
    , _mutex{}
{}

auto job_progress_t::job_id() const noexcept //
    -> job_id_t
{
    return _job_id;
}

auto job_progress_t::lock() const //
    -> std::unique_lock<std::mutex>
{
    return std::unique_lock{_mutex};
}

auto job_progress_t::status() const //
    -> job_status_e
{
    return _status;
}
auto job_progress_t::desc() const noexcept //
    -> const std::string&
{
    return _desc;
}
auto job_progress_t::num_steps() const noexcept //
    -> std::int16_t
{
    return _num_steps;
}

auto job_progress_t::status(job_status_e status) noexcept //
    -> void
{
    auto _ = lock();
    _status = std::move(status);
}

auto job_progress_t::desc(std::string desc) noexcept //
    -> void
{
    auto _ = lock();
    _desc = std::move(desc);
}

auto job_progress_t::num_steps(std::int16_t num_steps) noexcept //
    -> void
{
    auto _ = lock();
    _num_steps = std::move(num_steps);
}

auto job_progress_t::current_step() const noexcept //
    -> const current_step_t&
{
    return _current_step;
}

auto job_progress_t::next_step(std::string desc) //
    -> void
{
    return next_step(std::move(desc), {}, {});
}

auto job_progress_t::next_step(std::string desc, std::string unit, std::uint32_t units_total) //
    -> void
{
    auto _ = lock();
    _current_step._desc = std::move(desc);
    _current_step._rate = {};
    _current_step._unit = std::move(unit);
    _current_step._units_done = {};
    _current_step._units_total = std::move(units_total);
    _current_step._num++;
}

auto job_progress_t::result() const noexcept //
    -> const result_t&
{
    return _result;
}

auto job_progress_t::result(std::int32_t code) //
    -> void
{
    return result(std::move(code), {});
}
auto job_progress_t::result(std::int32_t code, std::string message) //
    -> void
{
    auto _ = lock();
    _result = result_t{std::move(code), std::move(message)};
}

auto to_json(json_t& j, const job_progress_t& progress) //
    -> void
{
    j = json_t{};

    auto lock = progress.lock();
    j["id"] = progress._job_id;
    j["status"] = to_string(progress._status);
    j["description"] = progress._desc;
    j["numSteps"] = progress._num_steps;
    j["currentStep"] = json_t::object();
    j["currentStep"]["description"] = progress._current_step._desc;
    j["currentStep"]["num"] = progress._current_step._num;
    j["currentStep"]["unit"] = progress._current_step._unit;
    j["currentStep"]["unitsTotal"] = progress._current_step._units_total;
    j["currentStep"]["unitsDone"] = progress._current_step._units_done;
    j["currentStep"]["rate"] = progress._current_step._rate;
    j["result"] = json_t::object();
    j["result"]["code"] = std::get<0>(progress._result);
    j["result"]["message"] = std::get<1>(progress._result);
}

auto operator<(const job_progress_t& lhs, const job_progress_t& rhs) //
    -> bool
{
    return lhs.job_id() < rhs.job_id();
}

auto operator<=(const job_progress_t& lhs, const job_progress_t& rhs) //
    -> bool
{
    return !(lhs > rhs);
}

auto operator>(const job_progress_t& lhs, const job_progress_t& rhs) //
    -> bool
{
    return rhs < lhs;
}

auto operator>=(const job_progress_t& lhs, const job_progress_t& rhs) //
    -> bool
{
    return !(lhs < rhs);
}

auto operator==(const job_progress_t& lhs, const job_progress_t& rhs) //
    -> bool
{
    return lhs.job_id() == rhs.job_id();
}

auto operator!=(const job_progress_t& lhs, const job_progress_t& rhs) //
    -> bool
{
    return !(lhs == rhs);
}

} // namespace flecs
