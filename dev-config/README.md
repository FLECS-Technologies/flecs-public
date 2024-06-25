# Configuration files for development
There is currently only one configuration file used during development.
## floxy/floxy.conf
This file is used by floxy, a nginx reverse proxy used to allow accessing the apps editors. This file should be linked or copied to `/etc/nginx/floxy.conf`. We cannot use the same configuration file because nginx accesses files that are not accessible to normal users. This modified configuration file sets the paths to these files to `/tmp/`.