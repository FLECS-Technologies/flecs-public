/*
 * Copyright (c) 2017, 2020 ADLINK Technology Inc.
 *
 * This program and the accompanying materials are made available under the
 * terms of the Eclipse Public License 2.0 which is available at
 * http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
 * which is available at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
 *
 * Contributors:
 *   ADLINK zenoh team, <zenoh@adlink-labs.tech>
 */


#ifndef ZENOH_NET_
#define ZENOH_NET_

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>


/**
 * The behavior to adopt in case of congestion while routing some data.
 *
 *     - **zn_congestion_control_t_BLOCK**
 *     - **zn_congestion_control_t_DROP**
 */
typedef enum zn_congestion_control_t {
  zn_congestion_control_t_BLOCK,
  zn_congestion_control_t_DROP,
} zn_congestion_control_t;

/**
 * The kind of consolidation that should be applied on replies to a :c:func:`zn_query`.
 *
 *     - **zn_consolidation_mode_t_FULL**: Guaranties unicity of replies. Optimizes bandwidth.
 *     - **zn_consolidation_mode_t_LAZY**: Does not garanty unicity. Optimizes latency.
 *     - **zn_consolidation_mode_t_NONE**: No consolidation.
 */
typedef enum zn_consolidation_mode_t {
  zn_consolidation_mode_t_FULL,
  zn_consolidation_mode_t_LAZY,
  zn_consolidation_mode_t_NONE,
} zn_consolidation_mode_t;

/**
 * The subscription reliability.
 *
 *     - **zn_reliability_t_BEST_EFFORT**
 *     - **zn_reliability_t_RELIABLE**
 */
typedef enum zn_reliability_t {
  zn_reliability_t_BEST_EFFORT,
  zn_reliability_t_RELIABLE,
} zn_reliability_t;

/**
 * The possible values of :c:member:`zn_reply_t.tag`
 *
 *     - **zn_reply_t_Tag_DATA**: The reply contains some data.
 *     - **zn_reply_t_Tag_FINAL**: The reply does not contain any data and indicates that there will be no more replies for this query.
 */
typedef enum zn_reply_t_Tag {
  zn_reply_t_Tag_DATA,
  zn_reply_t_Tag_FINAL,
} zn_reply_t_Tag;

/**
 * The subscription mode.
 *
 *     - **zn_submode_t_PUSH**
 *     - **zn_submode_t_PULL**
 */
typedef enum zn_submode_t {
  zn_submode_t_PUSH,
  zn_submode_t_PULL,
} zn_submode_t;

typedef struct zn_properties_t zn_properties_t;

typedef struct zn_publisher_t zn_publisher_t;

typedef struct zn_query_t zn_query_t;

typedef struct zn_queryable_t zn_queryable_t;

typedef struct zn_session_t zn_session_t;

typedef struct zn_subscriber_t zn_subscriber_t;

/**
 * A string.
 *
 * Members:
 *   const char *val: A pointer to the string.
 *   unsigned int len: The length of the string.
 *
 */
typedef struct z_string_t {
  const char *val;
  size_t len;
} z_string_t;

/**
 * A resource key.
 *
 * Resources are identified by URI like string names.
 * Examples : ``"/some/resource/key"``.
 * Resource names can be mapped to numerical ids through :c:func:`zn_declare_resource`
 * for wire and computation efficiency.
 *
 * A resource key can be either:
 *
 *   - A plain string resource name.
 *   - A pure numerical id.
 *   - The combination of a numerical prefix and a string suffix.
 *
 * Members:
 *   unsigned long id: The id or prefix of this resource key. ``0`` if empty.
 *   const char *suffix: The suffix of this resource key. ``NULL`` if pure numerical id.
 */
typedef struct zn_reskey_t {
  unsigned long id;
  const char *suffix;
} zn_reskey_t;

/**
 * The subscription period.
 *
 * Members:
 *     unsigned int origin:
 *     unsigned int period:
 *     unsigned int duration:
 */
typedef struct zn_period_t {
  unsigned int origin;
  unsigned int period;
  unsigned int duration;
} zn_period_t;

/**
 * Informations to be passed to :c:func:`zn_declare_subscriber` to configure the created :c:type:`zn_subscriber_t`.
 *
 * Members:
 *     zn_reliability_t reliability: The subscription reliability.
 *     zn_submode_t mode: The subscription mode.
 *     zn_period_t *period: The subscription period.
 */
typedef struct zn_subinfo_t {
  enum zn_reliability_t reliability;
  enum zn_submode_t mode;
  struct zn_period_t *period;
} zn_subinfo_t;

/**
 * An array of bytes.
 *
 * Members:
 *   const unsigned char *val: A pointer to the bytes array.
 *   unsigned int len: The length of the bytes array.
 *
 */
typedef struct z_bytes_t {
  const uint8_t *val;
  size_t len;
} z_bytes_t;

/**
 * A zenoh-net data sample.
 *
 * A sample is the value associated to a given resource at a given point in time.
 *
 * Members:
 *   z_string_t key: The resource key of this data sample.
 *   z_bytes_t value: The value of this data sample.
 */
typedef struct zn_sample_t {
  struct z_string_t key;
  struct z_bytes_t value;
} zn_sample_t;

/**
 * An array of NULL terminated strings.
 *
 * Members:
 *   char *const *val: A pointer to the array.
 *   unsigned int len: The length of the array.
 *
 */
typedef struct z_str_array_t {
  const char *const *val;
  size_t len;
} z_str_array_t;

/**
 * A hello message returned by a zenoh entity to a scout message sent with :c:func:`zn_scout`.
 *
 * Members:
 *   unsigned int whatami: The kind of zenoh entity.
 *   z_bytes_t pid: The peer id of the scouted entity (empty if absent).
 *   z_str_array_t locators: The locators of the scouted entity.
 *
 */
typedef struct zn_hello_t {
  unsigned int whatami;
  struct z_bytes_t pid;
  struct z_str_array_t locators;
} zn_hello_t;

/**
 * An array of :c:struct:`zn_hello_t` messages.
 *
 * Members:
 *   const zn_hello_t *val: A pointer to the array.
 *   unsigned int len: The length of the array.
 *
 */
typedef struct zn_hello_array_t {
  const struct zn_hello_t *val;
  size_t len;
} zn_hello_array_t;

/**
 * The possible values of :c:member:`zn_target_t.tag`.
 *
 *     - **zn_target_t_BEST_MATCHING**: The nearest complete queryable if any else all matching queryables.
 *     - **zn_target_t_COMPLETE**: A set of complete queryables.
 *     - **zn_target_t_ALL**: All matching queryables.
 *     - **zn_target_t_NONE**: No queryables.
 */
typedef enum zn_target_t_Tag {
  zn_target_t_BEST_MATCHING,
  zn_target_t_COMPLETE,
  zn_target_t_ALL,
  zn_target_t_NONE,
} zn_target_t_Tag;

typedef struct zn_target_t_COMPLETE_Body {
  unsigned int n;
} zn_target_t_COMPLETE_Body;

typedef struct zn_target_t {
  zn_target_t_Tag tag;
  union {
    zn_target_t_COMPLETE_Body complete;
  };
} zn_target_t;

/**
 * The zenoh-net queryables that should be target of a :c:func:`zn_query`.
 *
 * Members:
 *     unsigned int kind: A mask of queryable kinds.
 *     zn_target_t target: The query target.
 */
typedef struct zn_query_target_t {
  unsigned int kind;
  struct zn_target_t target;
} zn_query_target_t;

/**
 * The kind of consolidation that should be applied on replies to a :c:func:`zn_query`
 * at the different stages of the reply process.
 *
 * Members:
 *   zn_consolidation_mode_t first_routers: The consolidation mode to apply on first routers of the replies routing path.
 *   zn_consolidation_mode_t last_router: The consolidation mode to apply on last router of the replies routing path.
 *   zn_consolidation_mode_t reception: The consolidation mode to apply at reception of the replies.
 */
typedef struct zn_query_consolidation_t {
  enum zn_consolidation_mode_t first_routers;
  enum zn_consolidation_mode_t last_router;
  enum zn_consolidation_mode_t reception;
} zn_query_consolidation_t;

/**
 * An reply to a :c:func:`zn_query` (or :c:func:`zn_query_collect`).
 *
 * Members:
 *   zn_sample_t data: a :c:type:`zn_sample_t` containing the key and value of the reply.
 *   unsigned int source_kind: The kind of the replier that sent this reply.
 *   z_bytes_t replier_id: The id of the replier that sent this reply.
 *
 */
typedef struct zn_reply_data_t {
  struct zn_sample_t data;
  unsigned int source_kind;
  struct z_bytes_t replier_id;
} zn_reply_data_t;

/**
 * An reply to a :c:func:`zn_query`.
 *
 * Members:
 *   zn_reply_t_Tag tag: Indicates if the reply contains data or if it's a FINAL reply.
 *   zn_reply_data_t data: The reply data if :c:member:`zn_reply_t.tag` equals :c:member:`zn_reply_t_Tag.zn_reply_t_Tag_DATA`.
 *
 */
typedef struct zn_reply_t {
  enum zn_reply_t_Tag tag;
  struct zn_reply_data_t data;
} zn_reply_t;

/**
 * An array of :c:type:`zn_reply_data_t`.
 * Result of :c:func:`zn_query_collect`.
 *
 * Members:
 *   char *const *val: A pointer to the array.
 *   unsigned int len: The length of the array.
 *
 */
typedef struct zn_reply_data_array_t {
  const struct zn_reply_data_t *val;
  size_t len;
} zn_reply_data_array_t;

extern const unsigned int ZN_ROUTER;

extern const unsigned int ZN_PEER;

extern const unsigned int ZN_CLIENT;

extern const unsigned int ZN_QUERYABLE_ALL_KINDS;

extern const unsigned int ZN_QUERYABLE_STORAGE;

extern const unsigned int ZN_QUERYABLE_EVAL;

extern const unsigned int ZN_CONFIG_MODE_KEY;

extern const unsigned int ZN_CONFIG_PEER_KEY;

extern const unsigned int ZN_CONFIG_LISTENER_KEY;

extern const unsigned int ZN_CONFIG_USER_KEY;

extern const unsigned int ZN_CONFIG_PASSWORD_KEY;

extern const unsigned int ZN_CONFIG_MULTICAST_SCOUTING_KEY;

extern const unsigned int ZN_CONFIG_MULTICAST_INTERFACE_KEY;

extern const unsigned int ZN_CONFIG_MULTICAST_IPV4_ADDRESS_KEY;

extern const unsigned int ZN_CONFIG_SCOUTING_TIMEOUT_KEY;

extern const unsigned int ZN_CONFIG_SCOUTING_DELAY_KEY;

extern const unsigned int ZN_CONFIG_ADD_TIMESTAMP_KEY;

extern const unsigned int ZN_CONFIG_LOCAL_ROUTING_KEY;

extern const unsigned int ZN_INFO_PID_KEY;

extern const unsigned int ZN_INFO_PEER_PID_KEY;

extern const unsigned int ZN_INFO_ROUTER_PID_KEY;

/**
 * Initialise the zenoh runtime logger
 *
 */
void z_init_logger(void);

void z_string_free(struct z_string_t zs);

/**
 * Construct a :c:type:`z_string_t` from a NULL terminated string.
 * The content of the given string is copied.
 *
 * Parameters:
 *     s: The NULL terminated string.
 *
 * Returns:
 *     A new :c:type:`z_string_t`.
 */
struct z_string_t z_string_make(const char *s);

/**
 * Close a zenoh-net session.
 *
 * Parameters:
 *     session: A zenoh-net session.
 */
void zn_close(struct zn_session_t *session);

/**
 * Create a default set of properties for client mode zenoh-net session configuration.
 * If peer is not null, it is added to the configuration as remote peer.
 *
 * Parameters:
 *   peer: An optional peer locator.
 */
struct zn_properties_t *zn_config_client(char *peer);

/**
 * Create a default set of properties for zenoh-net session configuration.
 */
struct zn_properties_t *zn_config_default(void);

/**
 * Create an empty set of properties for zenoh-net session configuration.
 */
struct zn_properties_t *zn_config_empty(void);

/**
 * Create a set of properties for zenoh-net session configuration, parsing a file listing the properties
 * (1 "key=value" per line, comments starting with '#' character are allowed).
 * Returns null if parsing fails.
 *
 * Parameters:
 *   path: The path to the file (must be in UTF-8).
 */
struct zn_properties_t *zn_config_from_file(const char *path);

/**
 * Create a set of properties for zenoh-net session configuration, parsing a string listing the properties
 * in such format: "mode=client;peer=tcp/127.0.0.1:7447".
 */
struct zn_properties_t *zn_config_from_str(const char *s);

/**
 * Create a default set of properties for peer mode zenoh-net session configuration.
 */
struct zn_properties_t *zn_config_peer(void);

/**
 * Convert a set of properties into a string.
 *
 * Parameters:
 *     config: The set of properties.
 *
 * Returns:
 *     A keys/values string containing with such format: "key1=value1;key2=value2;...".
 */
struct z_string_t zn_config_to_str(struct zn_properties_t *config);

/**
 * Declare a :c:type:`zn_publisher_t` for the given resource key.
 *
 * Written resources that match the given key will only be sent on the network
 * if matching subscribers exist in the system.
 *
 * Parameters:
 *     session: The zenoh-net session.
 *     resource: The resource key to publish.
 *
 * Returns:
 *    The created :c:type:`zn_publisher_t` or null if the declaration failed.
 */
struct zn_publisher_t *zn_declare_publisher(struct zn_session_t *session,
                                            struct zn_reskey_t reskey);

/**
 * Declare a :c:type:`zn_queryable_t` for the given resource key.
 *
 * Parameters:
 *     session: The zenoh-net session.
 *     resource: The resource key the :c:type:`zn_queryable_t` will reply to.
 *     kind: The kind of :c:type:`zn_queryable_t`.
 *     callback: The callback function that will be called each time a matching query is received.
 *     arg: A pointer that will be passed to the **callback** on each call.
 *
 * Returns:
 *    The created :c:type:`zn_queryable_t` or null if the declaration failed.
 */
struct zn_queryable_t *zn_declare_queryable(struct zn_session_t *session,
                                            struct zn_reskey_t reskey,
                                            unsigned int kind,
                                            void (*callback)(struct zn_query_t*, const void*),
                                            void *arg);

/**
 * Associate a numerical id with the given resource key.
 *
 * This numerical id will be used on the network to save bandwidth and
 * ease the retrieval of the concerned resource in the routing tables.
 *
 * Parameters:
 *     session: The zenoh-net session.
 *     resource: The resource key to map to a numerical id.
 *
 * Returns:
 *     A numerical id.
 */
unsigned long zn_declare_resource(struct zn_session_t *session, struct zn_reskey_t reskey);

/**
 * Declare a :c:type:`zn_subscriber_t` for the given resource key.
 *
 * Parameters:
 *     session: The zenoh-net session.
 *     resource: The resource key to subscribe.
 *     sub_info: The :c:type:`zn_subinfo_t` to configure the :c:type:`zn_subscriber_t`.
 *     callback: The callback function that will be called each time a data matching the subscribed resource is received.
 *     arg: A pointer that will be passed to the **callback** on each call.
 *
 * Returns:
 *    The created :c:type:`zn_subscriber_t` or null if the declaration failed.
 */
struct zn_subscriber_t *zn_declare_subscriber(struct zn_session_t *session,
                                              struct zn_reskey_t reskey,
                                              struct zn_subinfo_t sub_info,
                                              void (*callback)(const struct zn_sample_t*, const void*),
                                              void *arg);

/**
 * Free an array of :c:struct:`zn_hello_t` messages and it's contained :c:struct:`zn_hello_t` messages recursively.
 *
 * Parameters:
 *     strs: The array of :c:struct:`zn_hello_t` messages to free.
 *
 */
void zn_hello_array_free(struct zn_hello_array_t hellos);

/**
 * Get informations about an zenoh-net session.
 *
 * Parameters:
 *     session: A zenoh-net session.
 *
 * Returns:
 *     A :c:type:`zn_properties_t` map containing informations on the given zenoh-net session.
 */
struct zn_properties_t *zn_info(struct zn_session_t *session);

/**
 * Get informations about an zenoh-net session.
 *
 * Parameters:
 *     session: A zenoh-net session.
 *
 * Returns:
 *     A keys/values string containing informations on the given zenoh-net session.
 *     The format of the string is: "key1=value1;key2=value2;...".
 */
struct z_string_t zn_info_as_str(struct zn_session_t *session);

/**
 * Open a zenoh-net session
 *
 * Parameters:
 *     config: A set of properties.
 *
 * Returns:
 *     The created zenoh-net session or null if the creation did not succeed.
 */
struct zn_session_t *zn_open(struct zn_properties_t *config);

/**
 * Free a set of properties.
 *
 * Parameters:
 *   ps: A pointer to the properties.
 */
void zn_properties_free(struct zn_properties_t *ps);

/**
 * Get the property with the given key from a properties map.
 *
 * Parameters:
 *     ps: A pointer to properties map.
 *     key: The key of the property.
 *
 * Returns:
 *     The value of the property with key ``key`` in properties map ``ps``.
 */
struct z_string_t zn_properties_get(struct zn_properties_t *ps, unsigned int key);

/**
 * Insert a property with a given key to a properties map.
 * If a property with the same key already exists in the properties map, it is replaced.
 *
 * Parameters:
 *   ps: A pointer to the properties map.
 *   key: The key of the property to add.
 *   value: The value of the property to add.
 *
 * Returns:
 *     A pointer to the updated properties map.
 */
struct zn_properties_t *zn_properties_insert(struct zn_properties_t *ps,
                                             unsigned long key,
                                             struct z_string_t value);

/**
 * Get the length of the given properties map.
 *
 * Parameters:
 *     ps: A pointer to the properties map.
 *
 * Returns:
 *     The length of the given properties map.
 */
unsigned int zn_properties_len(struct zn_properties_t *ps);

/**
 * Return a new empty map of properties.
 */
struct zn_properties_t *zn_properties_make(void);

/**
 * Pull data for a pull mode :c:type:`zn_subscriber_t`. The pulled data will be provided
 * by calling the **callback** function provided to the :c:func:`zn_declare_subscriber` function.
 *
 * Parameters:
 *     sub: The :c:type:`zn_subscriber_t` to pull from.
 */
void zn_pull(struct zn_subscriber_t *sub);

/**
 * Query data from the matching queryables in the system.
 * Replies are provided through a callback function.
 *
 * Parameters:
 *     session: The zenoh-net session.
 *     resource: The resource key to query.
 *     predicate: An indication to matching queryables about the queried data.
 *     target: The kind of queryables that should be target of this query.
 *     consolidation: The kind of consolidation that should be applied on replies.
 *     callback: The callback function that will be called on reception of replies for this query.
 *     arg: A pointer that will be passed to the **callback** on each call.
 */
void zn_query(struct zn_session_t *session,
              struct zn_reskey_t reskey,
              const char *predicate,
              struct zn_query_target_t target,
              struct zn_query_consolidation_t consolidation,
              void (*callback)(struct zn_reply_t, const void*),
              void *arg);

/**
 * Query data from the matching queryables in the system.
 * Replies are collected in an array.
 *
 * Parameters:
 *     session: The zenoh-net session.
 *     resource: The resource key to query.
 *     predicate: An indication to matching queryables about the queried data.
 *     target: The kind of queryables that should be target of this query.
 *     consolidation: The kind of consolidation that should be applied on replies.
 *
 * Returns:
 *    An array containing all the replies for this query.
 */
struct zn_reply_data_array_t zn_query_collect(struct zn_session_t *session,
                                              struct zn_reskey_t reskey,
                                              const char *predicate,
                                              struct zn_query_target_t target,
                                              struct zn_query_consolidation_t consolidation);

/**
 * Create a default :c:type:`zn_query_consolidation_t`.
 */
struct zn_query_consolidation_t zn_query_consolidation_default(void);

/**
 * Get the predicate of a received query.
 *
 * Parameters:
 *     query: The query.
 *
 * Returns:
 *     The predicate of the query.
 */
struct z_string_t zn_query_predicate(struct zn_query_t *query);

/**
 * Get the resource name of a received query.
 *
 * Parameters:
 *     query: The query.
 *
 * Returns:
 *     The resource name of the query.
 */
struct z_string_t zn_query_res_name(struct zn_query_t *query);

/**
 * Create a default :c:type:`zn_query_target_t`.
 */
struct zn_query_target_t zn_query_target_default(void);

/**
 * Free a :c:type:`zn_reply_data_array_t` and it's contained replies.
 *
 * Parameters:
 *     replies: The :c:type:`zn_reply_data_array_t` to free.
 *
 */
void zn_reply_data_array_free(struct zn_reply_data_array_t replies);

/**
 * Free a :c:type:`zn_reply_data_t` contained data and replier_id.
 *
 * Parameters:
 *     reply_data: The :c:type:`zn_reply_data_t` to free.
 *
 */
void zn_reply_data_free(struct zn_reply_data_t reply_data);

/**
 * Create a resource key from a resource id.
 *
 * Parameters:
 *     id: The resource id.
 *
 * Returns:
 *     A new resource key.
 */
struct zn_reskey_t zn_rid(unsigned long id);

/**
 * Create a resource key from a resource id and a suffix.
 *
 * Parameters:
 *     id: The resource id.
 *     suffix: The suffix.
 *
 * Returns:
 *     A new resource key.
 */
struct zn_reskey_t zn_rid_with_suffix(unsigned long id, const char *suffix);

/**
 * Create a resource key from a resource name.
 *
 * Parameters:
 *     id: The resource name.
 *
 * Returns:
 *     A new resource key.
 */
struct zn_reskey_t zn_rname(const char *name);

/**
 * Free a :c:type:`zn_sample_t` contained key and value.
 *
 * Parameters:
 *     sample: The :c:type:`zn_sample_t` to free.
 *
 */
void zn_sample_free(struct zn_sample_t sample);

/**
 * Scout for routers and/or peers.
 *
 * Parameters:
 *     what: A whatami bitmask of zenoh entities kind to scout for.
 *     config: A set of properties to configure the scouting.
 *     scout_period: The time that should be spent scouting before returnng the results.
 *
 * Returns:
 *     An array of :c:struct:`zn_hello_t` messages.
 */
struct zn_hello_array_t zn_scout(unsigned int what,
                                 struct zn_properties_t *config,
                                 unsigned long scout_period);

/**
 * Send a reply to a query.
 *
 * This function must be called inside of a Queryable callback passing the
 * query received as parameters of the callback function. This function can
 * be called multiple times to send multiple replies to a query. The reply
 * will be considered complete when the Queryable callback returns.
 *
 * Parameters:
 *     query: The query to reply to.
 *     key: The resource key of this reply.
 *     payload: The value of this reply.
 *     len: The length of the value of this reply.
 */
void zn_send_reply(struct zn_query_t *query,
                   const char *key,
                   const uint8_t *payload,
                   unsigned int len);

/**
 * Free an array of NULL terminated strings and it's contained NULL terminated strings recursively.
 *
 * Parameters:
 *     strs: The array of NULL terminated strings to free.
 *
 */
void zn_str_array_free(struct z_str_array_t strs);

/**
 * Create a default subscription info.
 */
struct zn_subinfo_t zn_subinfo_default(void);

/**
 * Create a default :c:type:`zn_target_t`.
 */
struct zn_target_t zn_target_default(void);

/**
 * Undeclare a :c:type:`zn_publisher_t`.
 *
 * Parameters:
 *     sub: The :c:type:`zn_publisher_t` to undeclare.
 */
void zn_undeclare_publisher(struct zn_publisher_t *publ);

/**
 * Undeclare a :c:type:`zn_queryable_t`.
 *
 * Parameters:
 *     qable: The :c:type:`zn_queryable_t` to undeclare.
 */
void zn_undeclare_queryable(struct zn_queryable_t *qable);

/**
 * Undeclare a :c:type:`zn_subscriber_t`.
 *
 * Parameters:
 *     sub: The :c:type:`zn_subscriber_t` to undeclare.
 */
void zn_undeclare_subscriber(struct zn_subscriber_t *sub);

/**
 * Write data.
 *
 * Parameters:
 *     session: The zenoh-net session.
 *     resource: The resource key to write.
 *     payload: The value to write.
 *     len: The length of the value to write.
 * Returns:
 *     ``0`` in case of success, ``1`` in case of failure.
 */
int zn_write(struct zn_session_t *session,
             struct zn_reskey_t reskey,
             const uint8_t *payload,
             unsigned int len);

/**
 * Write data with extended options.
 *
 * Parameters:
 *     session: The zenoh-net session.
 *     resource: The resource key to write.
 *     payload: The value to write.
 *     len: The length of the value to write.
 *     encoding: The encoding of the value.
 *     kind: The kind of value.
 *     congestion_control: The behavior to adopt in case of congestion while routing some data.
 * Returns:
 *     ``0`` in case of success, ``1`` in case of failure.
 */
int zn_write_ext(struct zn_session_t *session,
                 struct zn_reskey_t reskey,
                 const uint8_t *payload,
                 unsigned int len,
                 unsigned int encoding,
                 unsigned int kind,
                 enum zn_congestion_control_t congestion_control);

#endif /* ZENOH_NET_ */
