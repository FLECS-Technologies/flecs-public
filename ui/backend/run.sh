#!/bin/bash
sudo -u flecs strace -ff -v -s2048 -etrace=execve flecs-backend
