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

PACKAGE_DESC := FLECS Service Updater

DOCKER_IMAGE := marketplace.flecs.tech:5001/flecs/updater

include $(ROOT_DIR)/build/Makefile

docker-prepare:
	@tar czf usr/bin/flecs.deb.tar.gz -C $(ROOT_DIR)/daemon/out/$(MACHINE) flecs_$(VERSION)_$(ARCH).deb
	@tar czf usr/bin/flecs-backend.deb.tar.gz -C $(ROOT_DIR)/ui/backend/out/$(MACHINE) flecs-backend_$(VERSION)_$(ARCH).deb
	@tar czf usr/bin/flecs-updater.deb.tar.gz -C $(ROOT_DIR)/updater/out/$(MACHINE) flecs-updater_$(VERSION)_$(ARCH).deb
