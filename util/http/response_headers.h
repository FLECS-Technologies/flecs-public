// Copyright 2021 FLECS Technologies GmbH
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

#ifndef FLECS_util_http_response_headers_h
#define FLECS_util_http_response_headers_h

#include "util/container/map_constexpr.h"
#include "util/http/status_codes.h"

#include <array>
#include <utility>

namespace FLECS {

constexpr const char* http_200 = "200 OK\r\n";
// Client errors
constexpr const char* http_400 = "400 Bad Request\r\n";
constexpr const char* http_401 = "401 Unauthorized\r\n";
constexpr const char* http_402 = "402 Payment Required\r\n";
constexpr const char* http_403 = "403 Forbidden\r\n";
constexpr const char* http_404 = "404 Not Found\r\n";
constexpr const char* http_405 = "405 Method Not Allowed\r\n";
constexpr const char* http_406 = "406 Not Acceptable\r\n";
constexpr const char* http_407 = "407 Proxy Authentication Required\r\n";
constexpr const char* http_408 = "408 Request Timeout\r\n";
constexpr const char* http_409 = "409 Conflict\r\n";
constexpr const char* http_410 = "410 Gone\r\n";
constexpr const char* http_411 = "411 Length Required\r\n";
constexpr const char* http_412 = "412 Precondition Failed\r\n";
constexpr const char* http_413 = "413 Payload Too Large\r\n";
constexpr const char* http_414 = "414 URI Too Long\r\n";
constexpr const char* http_415 = "415 Unsupported Media Type\r\n";
constexpr const char* http_416 = "416 Range Not Satisfiable\r\n";
constexpr const char* http_417 = "417 Expectation Failed\r\n";
constexpr const char* http_418 = "418 I’m a teapot\r\n";
constexpr const char* http_421 = "421 Misdirected Request\r\n";
constexpr const char* http_422 = "422 Unprocessable Entity\r\n";
constexpr const char* http_423 = "423 Locked\r\n";
constexpr const char* http_424 = "424 Failed Dependency\r\n";
constexpr const char* http_425 = "425 Too Early\r\n";
constexpr const char* http_426 = "426 Upgrade Required\r\n";
constexpr const char* http_428 = "428 Precondition Required\r\n";
constexpr const char* http_429 = "429 Too Many Requests\r\n";
constexpr const char* http_431 = "431 Request Header Fields Too Large\r\n";
// Server errors
constexpr const char* http_500 = "500 Internal Server Error\r\n";
constexpr const char* http_501 = "501 Not Implemented\r\n";
constexpr const char* http_502 = "502 Bad Gateway\r\n";
constexpr const char* http_503 = "503 Service Unavailable\r\n";
constexpr const char* http_504 = "504 Gateway Timeout\r\n";
constexpr const char* http_505 = "505 HTTP Version not supported\r\n";
constexpr const char* http_506 = "506 Variant Also Negotiates\r\n";
constexpr const char* http_507 = "507 Insufficient Storage\r\n";
constexpr const char* http_508 = "508 Loop Detected\r\n";
constexpr const char* http_509 = "509 Bandwidth Limit Exceeded\r\n";
constexpr const char* http_510 = "510 Not Extended\r\n";
constexpr const char* http_511 = "511 Network Authentication Required\r\n";

using http_response_header_map_t = FLECS::map_c<http_status_e, const char*, 42>;

constexpr http_response_header_map_t http_response_header_map = {{
    std::make_pair(http_status_e::Ok,                      http_200),
    // Client errors
    std::make_pair(http_status_e::BadRequest,              http_400),
    std::make_pair(http_status_e::Unauthorized,            http_401),
    std::make_pair(http_status_e::PaymentRequired,         http_402),
    std::make_pair(http_status_e::Forbidden,               http_403),
    std::make_pair(http_status_e::NotFound,                http_404),
    std::make_pair(http_status_e::MethodNotAllowed,        http_405),
    std::make_pair(http_status_e::NotAcceptable,           http_406),
    std::make_pair(http_status_e::ProxyAuthRequired,       http_407),
    std::make_pair(http_status_e::RequestTimeout,          http_408),
    std::make_pair(http_status_e::Conflict,                http_409),
    std::make_pair(http_status_e::Gone,                    http_410),
    std::make_pair(http_status_e::LengthRequired,          http_411),
    std::make_pair(http_status_e::PreconditionFailed,      http_412),
    std::make_pair(http_status_e::PayloadTooLarge,         http_413),
    std::make_pair(http_status_e::UriTooLong,              http_414),
    std::make_pair(http_status_e::UnsupportedMediaType,    http_415),
    std::make_pair(http_status_e::RangeNotSatisfiable,     http_416),
    std::make_pair(http_status_e::ExpectationFailed,       http_417),
    std::make_pair(http_status_e::ImATeapot,               http_418),
    std::make_pair(http_status_e::MisdirectRequest,        http_421),
    std::make_pair(http_status_e::UnprocessableEntity,     http_422),
    std::make_pair(http_status_e::Locked,                  http_423),
    std::make_pair(http_status_e::FailedDependency,        http_424),
    std::make_pair(http_status_e::TooEarly,                http_425),
    std::make_pair(http_status_e::UpgradeRequired,         http_426),
    std::make_pair(http_status_e::PreconditionRequired,    http_428),
    std::make_pair(http_status_e::TooManyRequests,         http_429),
    std::make_pair(http_status_e::ReqHeaderFieldsTooLarge, http_431),
    // Server errors
    std::make_pair(http_status_e::InternalServerError,    http_500),
    std::make_pair(http_status_e::NotImplemented,         http_501),
    std::make_pair(http_status_e::BadGateway,             http_502),
    std::make_pair(http_status_e::ServiceUnavailable,     http_503),
    std::make_pair(http_status_e::GatewayTimeout,         http_504),
    std::make_pair(http_status_e::VersionNotSupported,    http_505),
    std::make_pair(http_status_e::VariantAlsoNegotiates,  http_506),
    std::make_pair(http_status_e::InsufficientStorage,    http_507),
    std::make_pair(http_status_e::LoopDetected,           http_508),
    std::make_pair(http_status_e::BandwidthLimitExceeded, http_509),
    std::make_pair(http_status_e::NotExtended,            http_510),
    std::make_pair(http_status_e::NetworkAuthRequired,    http_511),
}};

} // namespace FLECS

#endif // FLECS_util_http_response_headers_h
