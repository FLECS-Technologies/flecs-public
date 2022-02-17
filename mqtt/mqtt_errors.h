// Copyright 2021-2022 FLECS Technologies GmbH
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

#ifndef FLECS_mqtt_mqtt_errors_h
#define FLECS_mqtt_mqtt_errors_h

namespace FLECS {

enum mqtt_error_t : int
{
    MQTT_ERR_OK = 0,
    MQTT_ERR_NOMEM = 1,
    MQTT_ERR_INVALID = 3,
    MQTT_ERR_NOTCONN = 4,
    MQTT_ERR_UNKNOWN = 13,
    MQTT_ERR_OS = 14,
    MQTT_ERR_UTF8 = 18,
    MQTT_ERR_PACKET_TOO_LARGE = 25,
};

} // namespace FLECS

#endif // FLECS_mqtt_mqtt_errors_h
