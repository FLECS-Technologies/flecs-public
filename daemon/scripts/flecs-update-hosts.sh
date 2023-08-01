#!/bin/bash

# Copyright 2021-2023 FLECS Technologies GmbH
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

(echo "init" && docker events --filter 'type=network' --filter 'event=connect' --filter 'event=disconnect' --filter 'network=flecs') | \
while read line; do
	# delete existing FLECS block from /etc/hosts
	sed -i '/### BEGIN FLECS ###/,/### END FLECS ###/d' /etc/hosts

	# create a new FLECS block in /etc/hosts
	echo "### BEGIN FLECS ###" >>/etc/hosts

	# print all containers' IPv4 address and name on the flecs network, separated by '#'
	# as IPv4 addresses contain their according subnet, filter them out via sed: 172.21.0.2/16 -> 172.21.0.2
	# results in a list such as:
	# 172.21.0.2#flecs-abcd1234
	# 172.21.0.3#flecs-0123abcd
	ENTRIES=`docker network inspect -f '{{range.Containers}}{{.IPv4Address}}#{{println .Name}}{{end}}' flecs | sed -E 's,/[0-9]{2},,g'`
	for i in ${ENTRIES}; do
		# split each entry into IP...
		IP=`echo ${i} | cut -f1 -d '#'`

		# ...and container name
		CONTAINER=`echo ${i} | cut -f2 -d '#'`

		# collect aliases: older versions of Docker do not contain a container's hostname in the network aliases,
		# so build an array consisting of container name, hostname, all network aliases...
		ALIASES=(`echo ${CONTAINER}` `docker inspect -f '{{.Config.Hostname}}' ${CONTAINER}` `docker inspect -f '{{.NetworkSettings.Networks.flecs.Aliases}}' ${CONTAINER} | grep -oP '(?<=\[).*(?=\])'`);

		# ...and filter out duplicates with sort
		ALIASES=`echo ${ALIASES[*]} | tr ' ' '\n' | sort -u | tr '\n' ' '`

		for j in ${ALIASES}; do
			# create a hosts entry for each alias with the container's IP
			echo "${IP} ${j}" >>/etc/hosts
		done
	done
	echo "### END FLECS ###" >>/etc/hosts
done
