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

/*! @todo */
#ifndef FLECS_EXPORT
#define FLECS_EXPORT
#endif // FLECS_EXPORT

#include <stdbool.h>

#include "mqtt_errors.h"
#include "mqtt_message.h"

#ifndef __cplusplus
#define FLECS_MQTT_HOST "flecs-mqtt"
#define FLECS_MQTT_PORT 1883
#define FLECS_MQTT_KEEPALIVE 60
#endif // __cplusplus

#ifdef __cplusplus

#include <functional>
#include <memory>
#include <mutex>
#include <string>

namespace FLECS {

namespace impl {
class mqtt_client_t;
} // namespace impl

/*! DNS name of the default FLECS MQTT broker */
constexpr const char* MQTT_HOST = "flecs-mqtt";
/*! Port of the default FLECS MQTT broker */
constexpr const int MQTT_PORT = 1883;
/*! Default keepalive value in seconds */
constexpr const int MQTT_KEEPALIVE = 60;

class mqtt_client_t
{
public:
    /*! @brief Constructor
     */
    FLECS_EXPORT mqtt_client_t();

    /*! @brief Copy constructor (deleted)
     */
    FLECS_EXPORT mqtt_client_t(const mqtt_client_t&) = delete;

    /*! @brief Move constructor
     */
    FLECS_EXPORT mqtt_client_t(mqtt_client_t&& other);

    /*! @brief copy-assignment operator (deleted)
     */
    FLECS_EXPORT mqtt_client_t& operator=(const mqtt_client_t&) = delete;

    /*! @brief move-assignment operator
     */
    FLECS_EXPORT mqtt_client_t& operator=(mqtt_client_t&& other);

    /*! @brief Destructor
     */
    FLECS_EXPORT ~mqtt_client_t();

    /*! @brief Connect to the internal FLECS MQTT broker with default values.
     *
     * @sa MQTT_HOST
     * @sa MQTT_PORT
     * @sa MQTT_KEEPALIVE
     *
     * @return MQTT error code
     *      MQTT_ERR_OK on success
     *      MQTT_ERR_INVALID if input parameters were invalid
     *          - host == nullptr
     *          - port < 0
     *          - keepalive < 5
     *      MQTT_ERR_OS if any system called returned an error (check errno for more details)
     */
    FLECS_EXPORT int connect();

    /*! @brief Connect to a custom MQTT broker
     *
     * @param[in] host Broker hostname or IP address
     * @param[in] port Broker port
     * @param[in] keepalive Timeout between PING messages in seconds, if no messages are exchanged
     * with the broker
     *
     * @return MQTT error code
     *      MQTT_ERR_OK on success
     *      MQTT_ERR_INVALID if any input parameter is invalid
     *          - host == nullptr
     *          - port < 0
     *          - keepalive < 5
     *      MQTT_ERR_OS if any system called returned an error (check errno for more details)
     */
    FLECS_EXPORT int connect(const char* host, int port, int keepalive);

    /*! @brief Reconnect to the currently connected MQTT broker
     *
     * @return MQTT error code
     *      MQTT_ERR_OK on success
     *      MQTT_ERR_INVALID if client is not connected
     *      MQTT_ERR_NOMEM if not enough memory is available
     *      MQTT_ERR_OS if any system called returned an error (check errno for more details)
     */
    FLECS_EXPORT int reconnect();

    /*! @brief Disconnect from the currently connected MQTT broker
     *
     * @return MQTT error code
     *      MQTT_ERR_OK on success
     *      MQTT_ERR_NOTCONN if client is not connected
     */
    FLECS_EXPORT int disconnect();

    /*! @brief Returns whether the client is connected to a broker
     *
     * @return bool
     *      true if client is connected to a broker
     *      false otherwise
     */
    FLECS_EXPORT bool is_connected() const noexcept;

    /*! @brief Subscribe to an MQTT topic
     *
     * @param[in] sub Pattern to subscribe to
     * @param[in] qos Requested Quality-of-Service for this subscription
     *
     * @return MQTT error code
     *      MQTT_ERR_OK on success
     *      MQTT_ERR_INVALID if any input parameter is invalid
     *          - subscription pattern invalid
     *          - qos < 0 or qos > 2
     *      MQTT_ERR_NOMEM if not enough memory is available
     *      MQTT_ERR_NOTCONN if client is not connected
     *      MQTT_ERR_UTF8 if the topic name is not valid UTF-8
     *      MQTT_ERR_PACKET_TOO_LARGE if the resulting message is too large for the broker
     */
    FLECS_EXPORT int subscribe(const char* sub, int qos);

    /*! @brief Unsubscribe from an MQTT topic
     *
     * @param[in] sub Pattern to unsubscribe from
     *
     * @return MQTT error code
     *      MQTT_ERR_OK on success
     *      MQTT_ERR_INVALID if any input parameter is invalid
     *          - subscription pattern invalid
     *      MQTT_ERR_NOMEM if not enough memory is available
     *      MQTT_ERR_NOTCONN if client is not connected
     *      MQTT_ERR_UTF8 if the topic name is not valid UTF-8
     *      MQTT_ERR_PACKET_TOO_LARGE if the resulting message is too large for the broker
     */
    FLECS_EXPORT int unsubscribe(const char* sub);

    /*! @brief Publish a topic on the currently connected MQTT broker
     *
     * @param[in] topic Name of the topic to publish to
     * @param[in] payloadlen Size of the payload in bytes
     * @param[in] payload Pointer to the payload; must be valid if payloadlen > 0
     * @param[in] qos Quality-of-Service to use for this message
     * @param[in] retain True if message should be retained in the broker
     *
     * @return MQTT error code
     */
    FLECS_EXPORT int publish(
        const char* topic, int payloadlen, const void* payload, int qos, bool retain) const;
    FLECS_EXPORT int publish(
        const char* topic, int payloadlen, const void* payload, int qos, bool retain);

    /*! @brief Publish a topic on the currently connected MQTT broker
     *
     * @param[in] topic Name of the topic to publish to
     * @param[out] mid Message ID which has been assigned to this message
     * @param[in] payloadlen Size of the payload in bytes
     * @param[in] payload Pointer to the payload; must be valid if payloadlen > 0
     * @param[in] qos Quality-of-Service to use for this message
     * @param[in] retain True if message should be retained in the broker
     *
     * @return MQTT error code
     */
    FLECS_EXPORT int publish(
        const char* topic, int* mid, int payloadlen, const void* payload, int qos, bool retain)
        const;
    FLECS_EXPORT int publish(
        const char* topic, int* mid, int payloadlen, const void* payload, int qos, bool retain);

    /*! @brief Type for MQTT message callbacks
     *
     * @param[in] mqtt_client_t pointer to the mqtt_client_t instance that triggered the callback
     * @param[in] mqtt_message_t* Pointer to MQTT message. Message is only valid during execution of
     * the callback. If message is required after the callback, a copy has to be made by the user
     */
    using receive_cbk_t = std::function<void(mqtt_client_t*, mqtt_message_t*)>;

    /*! @brief Type for MQTT message callbacks with userdata
     *
     * @param[in] mqtt_client_t pointer to the mqtt_client_t instance that triggered the callback
     * @param[in] void* pointer to arbitrary userdata
     * @param[in] mqtt_message_t* Pointer to MQTT message. Message is only valid during execution of
     * the callback. If message is required after the callback, a copy has to be made by the user
     */
    using receive_cbk_userp_t = std::function<void(mqtt_client_t*, mqtt_message_t*, void*)>;

    /*! @brief Register a receive callback function on the client
     *
     * @param[in] cbk Function pointer to callback
     *
     * @return MQTT_ERR_OK
     */
    FLECS_EXPORT int receive_callback_set(receive_cbk_t cbk);

    /*! @brief Register a receive callback function on the client
     *
     * @param[in] cbk Function pointer to callback
     * @param[in] userp Optional userdata that is passed to the callback
     *
     * @return MQTT_ERR_OK
     */
    FLECS_EXPORT int receive_callback_set(receive_cbk_userp_t cbk, void* userp);

    /*! @brief Unregister the receive callback function on the client
     *
     * @param none
     *
     * @return MQTT_ERR_OK
     */
    FLECS_EXPORT int receive_callback_clear();

    /*! @brief Type for disconnect callbacks
     *
     * @param[in] mqtt_client_t pointer to the mqtt_client_t instance that triggered the callback
     */
    using disconnect_cbk_t = std::function<void(mqtt_client_t*)>;

    /*! @brief Type for disconnect callbacks with userdata
     *
     * @param[in] mqtt_client_t pointer to the mqtt_client_t instance that triggered the callback
     * @param[in] void* pointer to arbitrary userdata
     */
    using disconnect_cbk_userp_t = std::function<void(mqtt_client_t*, void*)>;

    /*! @brief Register a disconnect callback function on the client
     *
     * @param[in] cbk Function pointer to callback
     *
     * @return MQTT_ERR_OK
     */
    FLECS_EXPORT int disconnect_callback_set(disconnect_cbk_t cbk);

    /*! @brief Register a receive callback function on the client
     *
     * @param[in] cbk Function pointer to callback
     * @param[in] userp Optional userdata that is passed to the callback
     *
     * @return MQTT_ERR_OK
     */
    FLECS_EXPORT int disconnect_callback_set(disconnect_cbk_userp_t cbk, void* userp);

    /*! @brief Unregister the disconnect callback function on the client
     *
     * @param none
     *
     * @return MQTT_ERR_OK
     */
    FLECS_EXPORT int disconnect_callback_clear();

private:
    /*! friend swap function */
    friend FLECS_EXPORT void swap(mqtt_client_t& lhs, mqtt_client_t& rhs) noexcept;

    /*! Pointer to implementation */
    std::unique_ptr<impl::mqtt_client_t> _impl;
};

} // namespace FLECS

extern "C" {
#endif // __cplusplus

typedef void (*flecs_mqtt_callback)(void*, struct flecs_mqtt_message_t*, void*);

FLECS_EXPORT void* flecs_mqtt_client_new(void);

FLECS_EXPORT void flecs_mqtt_client_destroy(void* mqtt);

FLECS_EXPORT int flecs_mqtt_connect(void* mqtt, const char* host, int port, int keepalive);

FLECS_EXPORT int flecs_mqtt_reconnect(void* mqtt);

FLECS_EXPORT int flecs_mqtt_disconnect(void* mqtt);

FLECS_EXPORT bool flecs_mqtt_is_connected(void* mqtt);

FLECS_EXPORT int flecs_mqtt_subscribe(void* mqtt, const char* sub, int qos);

FLECS_EXPORT int flecs_mqtt_unsubscribe(void* mqtt, const char* sub);

FLECS_EXPORT int flecs_mqtt_publish(
    const void* mqtt, const char* topic, int payloadlen, const void* payload, int qos, bool retain);

FLECS_EXPORT int flecs_mqtt_publish_mid(
    const void* mqtt,
    const char* topic,
    int* mid,
    int payloadlen,
    const void* payload,
    int qos,
    bool retain);

FLECS_EXPORT int flecs_mqtt_receive_callback_set(void* mqtt, flecs_mqtt_callback cbk, void* userp);

FLECS_EXPORT int flecs_mqtt_receive_callback_clear(void* mqtt);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
