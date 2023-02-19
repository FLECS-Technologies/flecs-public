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

#include "flecsport.h"

#include "common/app/app_key.h"
#include "common/instance/instance_id.h"
#include "factory/factory.h"
#include "impl/flecsport_impl.h"
#include "util/datetime/datetime.h"

namespace FLECS {

namespace {
register_module_t<module_flecsport_t> _reg("flecsport");
}

module_flecsport_t::module_flecsport_t()
    : _impl{new impl::module_flecsport_t{this}}
{}

module_flecsport_t::~module_flecsport_t()
{}

auto module_flecsport_t::do_init() //
    -> void
{
    FLECS_V2_ROUTE("/exports/create").methods("POST"_method)([this](const crow::request& req) {
        auto response = json_t{};
        const auto args = parse_json(req.body);
        auto apps = std::vector<app_key_t>{};
        if (args.contains("apps")) {
            args["apps"].get_to(apps);
        }
        auto instances = std::vector<instance_id_t>{};
        if (args.contains("instances")) {
            args["instances"].get_to(instances);
        }
        if (apps.empty() && instances.empty()) {
            return crow::response{crow::status::BAD_REQUEST, "", {}};
        }
        return http_export_to(std::move(apps), std::move(instances));
    });

    return _impl->do_init();
}

auto module_flecsport_t::http_export_to(
    std::vector<app_key_t> apps, std::vector<instance_id_t> instances) //
    -> crow::response
{
    const auto now = unix_time(precision_e::seconds);
    auto dest_dir = fs::path{"/var/lib/flecs/exports"} / now;
    auto job_id =
        _impl->queue_export_to(std::move(apps), std::move(instances), std::move(dest_dir));
    return crow::response{
        crow::status::ACCEPTED,
        "json",
        "{\"jobId\":" + std::to_string(job_id) + "}"};
}

} // namespace FLECS
