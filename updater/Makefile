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

PACKAGE := flecs-updater

DEB_PKG := flecs-updater.deb

DOCKER_IMAGE := marketplace.flecs.tech:5001/flecs/updater

include $(ROOT_DIR)/build/Makefile

docker-prepare: daemon ui/backend
	@shar -VQfx $(ROOT_DIR)/daemon/out/$(MACHINE)/flecs-service.deb >usr/bin/flecs-service.deb.shar
	@shar -VQfx $(ROOT_DIR)/ui/backend/out/$(MACHINE)/flecs-backend.deb >usr/bin/flecs-backend.deb.shar
	@shar -VQfx $(ROOT_DIR)/updater/out/$(MACHINE)/flecs-updater.deb >usr/bin/flecs-updater.deb.shar
