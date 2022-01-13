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

BINARY := flecsd

PACKAGE := flecs

PACKAGE_DESC := FLECS

DEPENDS_BINARY := \
	util/curl_easy_ext/archive \
	util/process/archive \
	util/sqlite3_ext/archive

SRC_BINARY := \
	src/daemon.cpp \
	src/main.cpp \
	src/app/app.cpp \
	src/db/app_db.cpp \
	src/service/private/service_app_manager_private.cpp \
	src/service/service_app_manager.cpp \
	src/service/service.cpp \
	src/service/service_help.cpp \
	src/service/service_rpc.cpp \

HEADER_BINARY := \
	include/app/app.h \
	include/app/app_status.h \
	include/db/app_db.h \
	include/daemon.h \
	include/instance/instance_status.h \
	include/service/private/service_app_manager_private.h \
	include/service/service_app_manager.h \
	include/service/service_errors.h \
	include/service/service_factory.h \
	include/service/service.h \
	include/service/service_help.h \
	include/service/service_rpc.h \

INCLUDE_BINARY := \
	-I ./include \
	-I $(ROOT_DIR)/external/curl-7.80.0/include \
	-I $(ROOT_DIR)/external/jsoncpp-1.9.5/include \
	-I $(ROOT_DIR)/external/sqlite-3.37.0 \
	-I $(ROOT_DIR)/external/yaml-cpp-0.7.0/include \

STATIC_LIBS_BINARY = \
	$(ROOT_DIR)/util/process/out/$(MACHINE)/libFLECS.process.a \
	$(ROOT_DIR)/util/curl_easy_ext/out/$(MACHINE)/libFLECS.curl_easy_ext.a \
	$(ROOT_DIR)/util/sqlite3_ext/out/$(MACHINE)/libFLECS.sqlite3_ext.a \
	$(ROOT_DIR)/external/sqlite-3.37.0/out/$(MACHINE)/lib/libsqlite3.a \
	$(ROOT_DIR)/external/yaml-cpp-0.7.0/out/$(MACHINE)/libyaml-cpp.a \
	$(ROOT_DIR)/external/curl-7.80.0/out/$(MACHINE)/lib/libcurl.a \
	$(ROOT_DIR)/external/jsoncpp-1.9.5/out/$(MACHINE)/lib/libjsoncpp.a \
	$(ROOT_DIR)/external/openssl-1.1.1m/out/$(MACHINE)/lib/libssl.a \
	$(ROOT_DIR)/external/openssl-1.1.1m/out/$(MACHINE)/lib/libcrypto.a

DYNAMIC_LIBS_BINARY := \
	-ldl -lpthread -lstdc++fs

LIB := libFLECS.daemon.so

SRC_LIB := \
	src/lib/libflecs.cpp \
	src/lib/private/libflecs_private.cpp \

HEADER_LIB := \
	include/lib/libflecs.h \
	include/lib/private/libflecs_private.h \

INCLUDE_LIB := \
	-I $(ROOT_DIR)/daemon/include \

include $(ROOT_DIR)/build/Makefile

deb-pkg-prepare: cli
	@mkdir -p $(OUT)/debian/usr/bin
	@cp -pf $(ROOT_DIR)/cli/$(OUT)/flecs $(OUT)/debian/usr/bin/