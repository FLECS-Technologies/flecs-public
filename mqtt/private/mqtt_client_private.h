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

#ifndef DD4E43C0_03C7_429D_98EB_73BE4F3581D6
#define DD4E43C0_03C7_429D_98EB_73BE4F3581D6

#include <variant>

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

    /*! @brief returns internal flag that keeps track of broker connection *
     */
    bool is_connected() const noexcept;

    /*! @brief Forwards to mosquitto_subscribe(_mosq, nullptr, sub, qos)
     */
    int subscribe(const char* sub, int qos);

    /*! @brief Forwards to mosquitto_unsubscribe(_mosq, nullptr, sub)
     */
    int unsubscribe(const char* sub);

    /*! @brief Forwards to mosquitto_publish(_mosq, mid, topic, payloadlen, payload, qos, retain)
     */
    int publish(const char* topic, int* mid, int payloadlen, const void* payload, int qos, bool retain) const;

    int receive_callback_set(mqtt_client_t::mqtt_receive_callback_t cbk, void* client);
    int receive_callback_set(mqtt_client_t::mqtt_receive_callback_userp_t cbk, void* client, void* userp);
    int receive_callback_clear();

    int disconnect_callback_set(mqtt_client_t::mqtt_disconnect_callback_t cbk, void* client);
    int disconnect_callback_set(mqtt_client_t::mqtt_disconnect_callback_userp_t cbk, void* client, void* userp);
    int disconnect_callback_clear();

private:
    /*! Pointer to mosquitto MQTT implementation */
    mosquitto* _mosq;
    /*! Flag to check if connected to a broker */
    bool _connected;

    /*! Function pointer to receive callback */
    using receive_cbk_t = std::
        variant<std::nullptr_t, mqtt_client_t::mqtt_receive_callback_t, mqtt_client_t::mqtt_receive_callback_userp_t>;
    receive_cbk_t _rcv_cbk;
    /*! Pointer to mqtt_client_t associated with this instance */
    void* _rcv_cbk_client;
    /*! Pointer to userdata passed to receive callback */
    void* _rcv_cbk_userp;

    /*! Function pointer to disconnect callback */
    using disconnect_cbk_t = std::variant<
        std::nullptr_t,
        mqtt_client_t::mqtt_disconnect_callback_t,
        mqtt_client_t::mqtt_disconnect_callback_userp_t>;
    disconnect_cbk_t _disconnect_cbk;
    /*! Pointer to mqtt_client_t tassociated with this instance */
    void* _disconnect_cbk_client;
    /*! Pointer to userdata passed to disconnect callback */
    void* _disconnect_cbk_userp;

    /*! @brief Receive callback function registered with the underlying mosquitto client library.
     */
    static void lib_receive_callback(mosquitto*, void*, const mosquitto_message*);

    /*! @brief Connect callback function registered with the underlying mosquitto client library.
     */
    static void lib_connect_callback(mosquitto*, void*, int);

    /*! @brief Disconnect callback function registered with the underlying mosquitto client library.
     */
    static void lib_disconnect_callback(mosquitto*, void*, int);
};

} // namespace Private
} // namespace FLECS

#endif // DD4E43C0_03C7_429D_98EB_73BE4F3581D6
