# Copyright 2021 FLECS Technologies GmbH
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
# http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

ROOT_DIR=$(shell git rev-parse --show-toplevel)

include $(ROOT_DIR)/build/Makeconfig
include $(ROOT_DIR)/build/Makerules
include $(ROOT_DIR)/build/Makefile_external

.PHONY: service
service: build/service package/service docker/service

.PHONY: ui/backend
ui/backend: build/ui/backend package/ui/backend docker/ui/backend

.PHONY: util/curl_easy_ext
util/curl_easy_ext: build/util/curl_easy_ext

.PHONY: util/llhttp_ext
util/llhttp_ext: build/util/llhttp_ext

.PHONY: util/process
util/process: build/util/process

.PHONY: util/sqlite3_ext
util/sqlite3_ext: build/util/sqlite3_ext
