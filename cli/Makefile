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

DEPENDS_BINARY := \
	daemon/lib

SRC_BINARY := \
	src/main.cpp \

HEADER_BINARY := \

INCLUDE_BINARY = \
	-I ./include \
	-I $(ROOT_DIR)/daemon/include \

DYNAMIC_LIBS_BINARY = \
	-L $(ROOT_DIR)/daemon/out/$(MACHINE) -l FLECS.daemon

include $(ROOT_DIR)/build/Makefile
