//
// Copyright (c) 2022 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ZettaScale Zenoh Team, <zenoh@zettascale.tech>

#include <assert.h>
#include <stddef.h>
#include <stdio.h>
#include <string.h>

#include "zenoh.h"

#define URL "demo/example"

void test_session() {
    z_owned_config_t config = z_config_default();
    z_owned_session_t session = z_open(z_move(config));
    assert(z_check(session));
    z_drop(z_move(session));
    assert(!z_check(session));
    z_drop(z_move(session));
    assert(!z_check(session));
}

void test_publisher() {
    z_owned_config_t config = z_config_default();
    z_owned_session_t s = z_open(z_move(config));
    z_owned_publisher_t pub = z_declare_publisher(z_loan(s), z_keyexpr(URL), NULL);
    assert(z_check(pub));
    z_drop(z_move(pub));
    assert(!z_check(pub));
    z_drop(z_move(pub));
    assert(!z_check(pub));
    z_drop(z_move(s));
}

void test_keyexpr() {
    z_owned_keyexpr_t keyexpr = z_keyexpr_new(URL);
    assert(z_check(keyexpr));
    z_drop(z_move(keyexpr));
    assert(!z_check(keyexpr));
    z_drop(z_move(keyexpr));
    assert(!z_check(keyexpr));
}

void test_config() {
    z_owned_config_t config = z_config_default();
    assert(z_check(config));
    z_drop(z_move(config));
    assert(!z_check(config));
    z_drop(z_move(config));
    assert(!z_check(config));
}

void test_scouting_config() {
    z_owned_scouting_config_t config = z_scouting_config_default();
    assert(z_check(config));
    z_drop(z_move(config));
    assert(!z_check(config));
    z_drop(z_move(config));
    assert(!z_check(config));
}

void data_handler(const z_sample_t *sample, void *arg) {}

void test_pull_subscriber() {
    z_owned_config_t config = z_config_default();
    z_owned_session_t s = z_open(z_move(config));
    z_owned_closure_sample_t callback = z_closure(data_handler);
    z_owned_pull_subscriber_t sub = z_declare_pull_subscriber(z_loan(s), z_keyexpr(URL), z_move(callback), NULL);
    assert(z_check(sub));
    z_drop(z_move(sub));
    assert(!z_check(sub));
    z_drop(z_move(sub));
    assert(!z_check(sub));
    z_drop(z_move(s));
}

void test_subscriber()
{
    z_owned_config_t config = z_config_default();
    z_owned_session_t s = z_open(z_move(config));
    z_owned_closure_sample_t callback = z_closure(data_handler);
    z_owned_subscriber_t sub = z_declare_subscriber(z_loan(s), z_keyexpr(URL), z_move(callback), NULL);
    assert(z_check(sub));
    z_drop(z_move(sub));
    assert(!z_check(sub));
    z_drop(z_move(sub));
    assert(!z_check(sub));
    z_drop(z_move(s));
}

void query_handler(const z_query_t *query, void *context) {}

void test_queryable()
{
    z_owned_config_t config = z_config_default();
    z_owned_session_t s = z_open(z_move(config));
    z_owned_closure_query_t callback = z_closure(query_handler);
    z_owned_queryable_t queryable = z_declare_queryable(z_loan(s), z_keyexpr(URL), z_move(callback), NULL);
    assert(z_check(queryable));
    z_drop(z_move(queryable));
    assert(!z_check(queryable));
    z_drop(z_move(queryable));
    assert(!z_check(queryable));
    z_drop(z_move(s));
}

int main(int argc, char **argv) {

    test_session();
    test_publisher();
    test_keyexpr();
    test_config();
    test_scouting_config();
    test_pull_subscriber();
    test_subscriber();
    test_queryable();

    return 0;
}
