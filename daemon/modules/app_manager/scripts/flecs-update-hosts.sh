#!/bin/bash

# Copyright 2021-2022 FLECS Technologies GmbH
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

if [ ! -f "/etc/hosts.old" ]; then
	cp -f /etc/hosts /etc/hosts.old
fi

(echo "init" && docker events --filter 'type=network' --filter 'event=connect' --filter 'event=disconnect' --filter 'network=flecs') | \
while read line; do
	sed -i '/### BEGIN FLECS ###/,/### END FLECS ###/d' /etc/hosts
	echo "### BEGIN FLECS ###" >>/etc/hosts
	ENTRIES=`docker network inspect -f '{{range.Containers}}{{.IPv4Address}}#{{println .Name}}{{end}}' flecs | sed -E 's,/[0-9]{2},,g'`
	for i in ${ENTRIES}; do
		IP=`echo ${i} | cut -f1 -d '#'`
		CONTAINER=`echo ${i} | cut -f2 -d '#'`
		ALIASES=`docker inspect -f '{{.NetworkSettings.Networks.flecs.Aliases}}' ${CONTAINER} | grep -oP '(?<=\[).*(?=\])'`;
		for j in ${ALIASES}; do
			echo "${IP} ${j}" >>/etc/hosts
		done
		echo "${IP} ${CONTAINER}" >>/etc/hosts
	done
	echo "### END FLECS ###" >>/etc/hosts
done
