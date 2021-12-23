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

ROOT_DIR := $(shell git rev-parse --show-toplevel)

BINARY := flecs-backend

DEPENDS_BINARY := \
	util/curl_easy_ext \
	util/llhttp_ext \
	util/process

SRC_BINARY := \
	src/main.cpp \
	src/http_request_handler.cpp

HEADER_BINARY := \
	http_request_handler.h

STATIC_LIBS_BINARY = \
	$(ROOT_DIR)/util/curl_easy_ext/out/$(MACHINE)/libFLECS.curl_easy_ext.a \
	$(ROOT_DIR)/util/llhttp_ext/out/$(MACHINE)/libFLECS.llhttp_ext.a \
	$(ROOT_DIR)/util/process/out/$(MACHINE)/libFLECS.process.a \
	$(ROOT_DIR)/external/jsoncpp-1.9.5/out/$(MACHINE)/lib/libjsoncpp.a

INCLUDE_BINARY = \
	-I $(ROOT_DIR)/external/jsoncpp-1.9.5/include \
	-I $(ROOT_DIR)/external/llhttp-2.1.4/out/$(MACHINE)/include

DYNAMIC_LIBS_BINARY := \
	-lpthread

DEB_PKG := flecs-backend.deb

DOCKER_IMAGE = marketplace.flecs.tech:5001/flecs/backend

include $(ROOT_DIR)/build/Makefile
