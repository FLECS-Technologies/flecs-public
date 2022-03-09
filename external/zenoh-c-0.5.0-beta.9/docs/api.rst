..
.. Copyright (c) 2017, 2020 ADLINK Technology Inc.
..
.. This program and the accompanying materials are made available under the
.. terms of the Eclipse Public License 2.0 which is available at
.. http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
.. which is available at https://www.apache.org/licenses/LICENSE-2.0.
..
.. SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
..
.. Contributors:
..   ADLINK zenoh team, <zenoh@adlink-labs.tech>
..

*************
API Reference
*************

Types
=====

String
------

.. autocstruct:: zenoh/net.h::z_string_t

.. autocfunction:: zenoh/net.h::z_string_make


Array of Str
------------

.. autocstruct:: zenoh/net.h::z_str_array_t

Bytes
-----

.. autocstruct:: zenoh/net.h::z_bytes_t

Properties
----------

.. c:type:: zn_properties_t

  A map of key/value properties where the key is an ``unsigned int``
  and the value a :c:type:`z_string_t`. Multiple values are coma separated.

.. autocfunction:: zenoh/net.h::zn_properties_make

.. autocfunction:: zenoh/net.h::zn_properties_len

.. autocfunction:: zenoh/net.h::zn_properties_insert

.. autocfunction:: zenoh/net.h::zn_properties_get

.. autocfunction:: zenoh/net.h::zn_properties_free

Scouting
========

Types
-----

Possible flags in a whatami bitmask : 

  .. c:var:: const unsigned int ZN_ROUTER

  .. c:var:: const unsigned int ZN_PEER

  .. c:var:: const unsigned int ZN_CLIENT

.. autocstruct:: zenoh/net.h::zn_hello_t

.. autocstruct:: zenoh/net.h::zn_hello_array_t

Functions
---------

.. autocfunction:: zenoh/net.h::zn_scout

.. autocfunction:: zenoh/net.h::zn_hello_array_free

Session
=======

Session configuration
---------------------

A zenoh-net session is configured through a :c:type:`zn_properties_t` properties map.

Multiple values are coma separated.

The following constants define the several property keys accepted for a zenoh-net 
session configuration and the associated accepted values.

.. c:var:: const unsigned int ZN_CONFIG_MODE_KEY

  The library mode.

    - Accepted values : ``"peer"``, ``"client"``.
    - Default value : ``"peer"``.

.. c:var:: const unsigned int ZN_CONFIG_PEER_KEY

  The locator of a peer to connect to.
    - Accepted values : ``<locator>`` (ex: ``"tcp/10.10.10.10:7447"``).
    - Default value : None.
    - Multiple values accepted.

.. c:var:: const unsigned int ZN_CONFIG_LISTENER_KEY

  A locator to listen on.

    - Accepted values : ``<locator>`` (ex: ``"tcp/10.10.10.10:7447"``).
    - Default value : None.
    - Multiple values accepted.

.. c:var:: const unsigned int ZN_CONFIG_USER_KEY

  The user name to use for authentication.

    - Accepted values : ``<string>``.
    - Default value : None.

.. c:var:: const unsigned int ZN_CONFIG_PASSWORD_KEY

  The password to use for authentication.

    - Accepted values : ``<string>``.
    - Default value : None.


.. c:var:: const unsigned int ZN_CONFIG_MULTICAST_SCOUTING_KEY

  Activates/Desactivates multicast scouting.

    - Accepted values : ``"true"``, ``"false"``.
    - Default value : ``"true"``.

.. c:var:: const unsigned int ZN_CONFIG_MULTICAST_INTERFACE_KEY

  The network interface to use for multicast scouting.

    - Accepted values : ``"auto"``, ``<ip address>``, ``<interface name>``.
    - Default value : ``"auto"``.

.. c:var:: const unsigned int ZN_CONFIG_MULTICAST_ADDRESS_KEY

  The multicast address and ports to use for multicast scouting.

    - Accepted values : ``<ip address>:<port>``.
    - Default value : ``"224.0.0.224:7447"``.

.. c:var:: const unsigned int ZN_CONFIG_SCOUTING_TIMEOUT_KEY

  In client mode, the period dedicated to scouting a router before failing.

    - Accepted values : ``<float in seconds>``.
    - Default value : ``"3.0"``.

.. c:var:: const unsigned int ZN_CONFIG_SCOUTING_DELAY_KEY

  In peer mode, the period dedicated to scouting first remote peers before doing anything else.

    - Accepted values : ``<float in seconds>``.
    - Default value : ``"0.2"``.

.. c:var:: const unsigned int ZN_CONFIG_ADD_TIMESTAMP_KEY

  Indicates if data messages should be timestamped.

    - Accepted values : ``"true"``, ``"false"``.
    - Default value : ``"false"``.

.. c:var:: const unsigned int ZN_CONFIG_LOCAL_ROUTING_KEY

  Indicates if local writes/queries should reach local subscribers/queryables.

    - Accepted values : ``"true"``, ``"false"``.
    - Default value : ``"true"``.

The following functions allow to create default :c:type:`zn_properties_t` maps for 
zenoh-net session configuration. The returned configurations can be amended with extra 
options with :c:func:`zn_properties_insert`.

.. autocfunction:: zenoh/net.h::zn_config_empty

.. autocfunction:: zenoh/net.h::zn_config_default

.. autocfunction:: zenoh/net.h::zn_config_peer

.. autocfunction:: zenoh/net.h::zn_config_client

Session management
------------------

.. autocfunction:: zenoh/net.h::zn_open

.. autocfunction:: zenoh/net.h::zn_info

.. autocfunction:: zenoh/net.h::zn_close

Resource
========

Resource key
------------

.. autocstruct:: zenoh/net.h::zn_reskey_t

.. autocfunction:: zenoh/net.h::zn_rname

.. autocfunction:: zenoh/net.h::zn_rid

.. autocfunction:: zenoh/net.h::zn_rid_with_suffix

Sample
------

.. autocstruct:: zenoh/net.h::zn_sample_t

.. autocfunction:: zenoh/net.h::zn_sample_free

Resource declaration
--------------------

.. autocfunction:: zenoh/net.h::zn_declare_resource

Publication
===========

Types
-----

.. c:type:: zn_publisher_tr

  A zenoh-net Publisher.

.. autocenum:: zenoh/net.h::zn_congestion_control_t

Functions
---------

.. autocfunction:: zenoh/net.h::zn_declare_publisher

.. autocfunction:: zenoh/net.h::zn_undeclare_publisher

.. autocfunction:: zenoh/net.h::zn_write

.. autocfunction:: zenoh/net.h::zn_write_ext

Subscription
============

Types
-----

.. c:type:: zn_subscriber_t

  A zenoh-net subscriber.

.. autocenum:: zenoh/net.h::zn_reliability_t

.. autocenum:: zenoh/net.h::zn_submode_t

.. autocstruct:: zenoh/net.h::zn_period_t

.. autocstruct:: zenoh/net.h::zn_subinfo_t

.. autocfunction:: zenoh/net.h::zn_subinfo_default

Functions
---------

.. autocfunction:: zenoh/net.h::zn_declare_subscriber

.. autocfunction:: zenoh/net.h::zn_pull

.. autocfunction:: zenoh/net.h::zn_undeclare_subscriber

Query
=====

Types
-----

.. c:struct:: zn_target_t

  Which amongst the matching queryables should be target of a :c:func:`zn_query`.

  .. c:member:: zn_target_t_Tag tag;

  .. c:member:: zn_target_t_COMPLETE_Body complete;

    Members of zn_target_t when :c:member:`zn_target_t.tag` is set to ``zn_target_t_COMPLETE``.

    .. c:member:: unsigned int n

      The number of complete queryables that should be target of a :c:func:`zn_query`.

.. autocenum:: zenoh/net.h::zn_target_t_Tag

.. autocfunction:: zenoh/net.h::zn_target_default

.. autocstruct:: zenoh/net.h::zn_query_target_t

  Predefined values for :c:member:`zn_query_target_t.kind`: 

    .. c:var:: const unsigned int ZN_QUERYABLE_ALL_KINDS

    .. c:var:: const unsigned int ZN_QUERYABLE_EVAL

    .. c:var:: const unsigned int ZN_QUERYABLE_STORAGE

.. autocfunction:: zenoh/net.h::zn_query_target_default

.. autocenum:: zenoh/net.h::zn_consolidation_mode_t

.. autocstruct:: zenoh/net.h::zn_query_consolidation_t

.. autocfunction:: zenoh/net.h::zn_query_consolidation_default

.. autocstruct:: zenoh/net.h::zn_reply_data_t

.. autocfunction:: zenoh/net.h::zn_reply_data_free

.. autocstruct:: zenoh/net.h::zn_reply_data_array_t

.. autocfunction:: zenoh/net.h::zn_reply_data_array_free

.. autocstruct:: zenoh/net.h::zn_reply_t

.. autocenum:: zenoh/net.h::zn_reply_t_Tag

Functions
---------

.. autocfunction:: zenoh/net.h::zn_query

.. autocfunction:: zenoh/net.h::zn_query_collect

Queryable
=========

Types
-----

.. c:type:: zn_queryable_t

  The zenoh-net Queryable.

.. c:type:: zn_query_t

  A query received by a Queryable. 

.. autocfunction:: zenoh/net.h::zn_query_res_name

.. autocfunction:: zenoh/net.h::zn_query_predicate


Functions
---------

.. autocfunction:: zenoh/net.h::zn_declare_queryable

  Predefined values for ``kind``: 

    .. c:var:: const unsigned int ZN_QUERYABLE_EVAL

    .. c:var:: const unsigned int ZN_QUERYABLE_STORAGE

.. autocfunction:: zenoh/net.h::zn_send_reply

.. autocfunction:: zenoh/net.h::zn_undeclare_queryable


