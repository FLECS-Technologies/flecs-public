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

********
Examples
********

Publish
=======

.. code-block:: c

  #include <string.h>
  #include "zenoh/net.h"

  int main(int argc, char **argv) {
      char* value = "value";

      zn_session_t *s = zn_open(zn_config_default());
      zn_write(s, zn_rname("/res/name"), (const uint8_t *)value, strlen(value));
      zn_close(s);

      return 0;
  }

Subscribe
=========

.. code-block:: c

  #include <stdio.h>
  #include "zenoh/net.h"

  void data_handler(const zn_sample_t *sample, const void *arg) {
      printf(">> Received (%.*s, %.*s)\n",
          (int)sample->key.len, sample->key.val,
          (int)sample->value.len, sample->value.val);
  }

  int main(int argc, char **argv) {
      zn_session_t *s = zn_open(zn_config_default());
      zn_subscriber_t *sub = zn_declare_subscriber(s, zn_rname("/res/name"), zn_subinfo_default(), data_handler, NULL);

      char c = 0;
      while (c != 'q') {
          c = fgetc(stdin);
      }

      zn_undeclare_subscriber(sub);
      zn_close(s);
      return 0;
  }

Query
=====

.. code-block:: c

  #include <stdio.h>
  #include <unistd.h>
  #include <string.h>
  #include "zenoh/net.h"

  int main(int argc, char** argv) {
      zn_session_t *s = zn_open(zn_config_default());
      zn_reply_data_array_t replies = zn_query_collect(s, zn_rname("/res/name"), "", zn_query_target_default(), zn_query_consolidation_default());
      
      for(unsigned int i = 0; i < replies.len; ++i) {
          printf(">> Received (%.*s, %.*s)\n",
            (int)replies.val[i].data.key.len, replies.val[i].data.key.val,
            (int)replies.val[i].data.value.len, replies.val[i].data.value.val);
      }
      zn_reply_data_array_free(replies);

      zn_close(s);
      return 0;
  }