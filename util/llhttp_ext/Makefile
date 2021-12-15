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

ARCHIVE := libFLECS.llhttp_ext.a

SRC := \
	src/llhttp_ext.cpp

HEADER := \
	llhttp_ext.h

INCLUDE = \
	-I $(ROOT_DIR) \
	-I $(ROOT_DIR)/external/curl-7.80.0/include \
	-I $(ROOT_DIR)/external/llhttp-2.1.4/out/$(MACHINE)/include

include $(ROOT_DIR)/build/Makefile

all: $(OUT)/$(ARCHIVE)

$(OUT)/$(ARCHIVE): $(DEPENDS) $(OBJ) $(HEADER)
	cp $(ROOT_DIR)/external/llhttp-2.1.4/out/$(MACHINE)/lib/libllhttp.a $@
	$(AR) r $@ $(OBJ)
