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

#ifndef FLECS_mqtt_mqtt_client_private_h
#define FLECS_mqtt_mqtt_client_private_h

#include <mutex>

#include "mqtt_client.h"

struct mosquitto;
struct mosquitto_message;

namespace FLECS {
namespace Private {

class mqtt_client_private_t
{
public:
    /*! @brief Constructor. On creation of the first instance, the underlying mosquitto MQTT library is initialized.
     */
    mqtt_client_private_t();

    /*! @brief Destructor. If the last instance is destroyed, the underlying mosquitto MQTT library is de-initialized.
     */
    ~mqtt_client_private_t();

    /*! @brief Forwards to mosquitto_connect(_mosq, host, port, keepalive)
     */
    int connect(const char* host, int port, int keepalive);

    /*! @brief Forwards to mosquitto_reconnect(_mosq)
     */

    /*! String holding the client identification */
    std::string _client_id;
    int reconnect();

    /*! @brief Forwards to mosquitto_disconnect(_mosq)
     */
    int disconnect();

    /*! @brief Forwards to mosquitto_subscribe(_mosq, nullptr, sub, qos)
     */
    int subscribe(const char* sub, int qos);

    /*! @brief Forwards to mosquitto_unsubscribe(_mosq, nullptr, sub)
     */
    int unsubscribe(const char* sub);

    /*! @brief Forwards to mosquitto_publish(_mosq, nullptr, topic, payloadlen, payload, qos, retain)
     */
    int publish(const char* topic, int payloadlen, const void* payload, int qos, bool retain);

    int receive_callback_set(mqtt_client_t::mqtt_callback_t cbk, void* client, void* userp);

    int receive_callback_clear();

    mqtt_client_t::mqtt_callback_t receive_callback();

private:
    /*! Pointer to mosquitto MQTT implementation */
    mosquitto* _mosq;

    /*! Function pointer to receive callback */
    mqtt_client_t::mqtt_callback_t _receive_cbk;
    /*! Pointer to mqtt_client_t that associated with this instance */
    void* _receive_cbk_client;
    /*! Pointer to userdata passed to receive callback */
    void* _receive_cbk_userp;

    /* @brief Receive callback function registered with the underlying mosquitto client library.
     */
    static void lib_receive_callback(mosquitto* _mosq, void* mqtt_client, const mosquitto_message* msg);

    /*! Number of currently instantiated clients */
    inline static int _ref_count{};
    /*! Internal mutex to protect non-MT-safe operations on the broker */
    inline static std::mutex _ref_mutex{};
};

} // namespace Private
} // namespace FLECS

#endif // FLECS_mqtt_mqtt_client_private_h
