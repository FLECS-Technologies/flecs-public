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

ARCHIVE := libFLECS.sqlite3_ext.a

SRC_ARCHIVE := \
	src/sqlite3_db.cpp \
	src/sqlite3_table.cpp \

HEADER_ARCHIVE := \
	include/sqlite3_column.h \
	include/sqlite3_db.h \
	include/sqlite3_primary.h \
	include/sqlite3_statements.h \
	include/sqlite3_table.h \

INCLUDE_ARCHIVE = \
	-I include -I $(ROOT_DIR)/external/sqlite-3.37.0/out/$(MACHINE)/include \

include $(ROOT_DIR)/build/Makefile
