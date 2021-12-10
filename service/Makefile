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

BINARY := flecs

DEPENDS := \
	external/curl \
	external/jsoncpp \
	external/sqlite \
	external/yaml-cpp \
	util/curl_easy_ext \
	util/process \
	util/sqlite3_ext

SRC := \
	main.cpp \
	src/app.cpp \
	src/app_db.cpp \
	src/service_app_manager.cpp \
	src/service_help.cpp \
	src/service_rpc.cpp \
	src/service.cpp \
	src/private/service_app_manager_private.cpp

HEADER := \
	app_db.h \
	service_app_manager.h \
	service_factory.h \
	service_help.h \
	service_rpc.h \
	service_table.h \
	service.h \
	private/service_app_manager_private.h \

INCLUDE := \
	-I $(ROOT_DIR)/external/sqlite-3.37.0 \
	-I $(ROOT_DIR)/external/yaml-cpp-0.7.0/include \
	-I $(ROOT_DIR)/external/curl-7.80.0/include

STATIC_LIBS = \
	$(ROOT_DIR)/util/process/out/$(MACHINE)/libFLECS.process.a \
	$(ROOT_DIR)/util/curl_easy_ext/out/$(MACHINE)/libFLECS.curl_easy_ext.a \
	$(ROOT_DIR)/util/sqlite3_ext/out/$(MACHINE)/libFLECS.sqlite3_ext.a \
	$(ROOT_DIR)/external/sqlite-3.37.0/out/$(MACHINE)/lib/libsqlite3.a \
	$(ROOT_DIR)/external/yaml-cpp-0.7.0/out/$(MACHINE)/libyaml-cpp.a \
	$(ROOT_DIR)/external/curl-7.80.0/out/$(MACHINE)/lib/libcurl.a \
	$(ROOT_DIR)/external/jsoncpp-1.9.5/out/$(MACHINE)/lib/libjsoncpp.a \
	$(ROOT_DIR)/external/openssl-1.1.1l/out/$(MACHINE)/lib/libssl.a \
	$(ROOT_DIR)/external/openssl-1.1.1l/out/$(MACHINE)/lib/libcrypto.a

DYNAMIC_LIBS := \
	-ldl -lpthread

include $(ROOT_DIR)/build/Makefile

.PHONY: all
all: $(OUT)/$(BINARY)

.PHONY: deb-pkg
deb-pkg:
	@mkdir -p debian/usr/bin
	@mkdir -p debian/var/lib/flecs/db
	@cp -pf $(OUT)/$(BINARY) debian/usr/bin/
	@touch debian/var/lib/flecs/db/apps.db
	@dpkg-deb --root-owner-group --build -Z gzip debian $(OUT)/flecs.deb