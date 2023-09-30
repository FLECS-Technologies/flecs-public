#!/bin/bash
# Copyright 2021-2023 FLECS Technologies GmbH
#
# Licensed under the Apache License, Version 2.  (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
# http://www.apache.org/licenses/LICENSE-2.
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

cat <<'EOF' > /tmp/filip.sh
#!/bin/bash
ME="FILiP"
SCRIPTNAME=`readlink -f ${0}`
ARGS="$*"
ROOT_DIR=/
STDOUT=/dev/null
STDERR=/dev/null

BASE_URL=dl.flecs.tech

print_usage() {
  echo "Usage: ${SCRIPTNAME}" [options]
  echo
  echo "  -d --debug           print additional debug messages"
  echo "  -y --yes             assume yes as answer to all prompts (unattended mode)"
  echo "     --no-banner       do not print ${ME} banner"
  echo "     --no-welcome      do not print welcome message"
  echo "     --root-dir <dir>  install files relative do <dir> instead of / (currently .tar only)"
  echo "     --help            print this help and exit"
}

# some log functions...
log_debug() {
  if [ ! -z "${LOG_DEBUG}" ]; then
    while true; do
      case ${1} in
        -n)
          local ECHO_ARGS="-n"
          shift
          ;;
        -q)
          local NO_PREFIX=true
          shift
          ;;
        *)
          break;;
      esac
    done
    if [ -z "${NO_PREFIX}" ]; then
      echo ${ECHO_ARGS} "*** Debug: $@"
    else
      echo ${ECHO_ARGS} "$@"
    fi
  fi
}
log_info() {
  while true; do
    case ${1} in
      -n)
        local ECHO_ARGS="-n"
        shift
        ;;
      -q)
        local NO_PREFIX=true
        shift
        ;;
      *)
        break;;
    esac
  done
  if [ -z "${NO_PREFIX}" ]; then
    echo ${ECHO_ARGS} "Info: $@"
  else
    echo ${ECHO_ARGS} "$@"
  fi
}
log_warning() {
  if [ -z "$@" ]; then
    echo 1>&2
  else
    echo "Warning: $@" 1>&2
  fi
}
log_error() {
  while true; do
    case ${1} in
      -n)
        local ECHO_ARGS="-n"
        shift
        ;;
      -q)
        local NO_PREFIX=true
        shift
        ;;
      *)
        break;;
    esac
  done
  if [ -z "${NO_PREFIX}" ]; then
    echo ${ECHO_ARGS} "Error: $@" 1>&2
  else
    echo ${ECHO_ARGS} "$@" 1>&2
  fi
}
# log_fatal will terminate with exit code 1 after logging
log_fatal() {
  if [ -z "$@" ]; then
    echo 1>&2
  else
    echo "Fatal: $@. ${ME} out." 1>&2
  fi
  exit 1
}
# internal_error should *only* be called if guaranteed preconditions are not met
internal_error() {
  log_error "Internal error: $@" 1>&2
  exit -1
}

# print a message and wait for user input. does nothing in unattended mode
confirm() {
  if [ -z "${ASSUME_YES}" ]; then
    read -s -p "$@"
  fi
}
confirm_yn() {
  if [ -z "${ASSUME_YES}" ]; then
    while true; do
      read -p "$*? [y/n]: " input
      case ${input} in
        [yY]*)
          return 0
          ;;
        [nN]*)
          return 1
          ;;
      esac
    done
  else
    return 0
  fi
}

# compare two version numbers in a robust way
cmp_less() {
  if [ -z "${1}" ] || [ -z "${2}" ]; then
    internal_error "attempt to compare with empty value: ${1} < ${2}"
  fi
  if [ "${1}" = "${2}" ]; then
    return 1
  fi
  local RES=`${SORT} -t . -k 1,1n -k 2,2n -k 3,3n <(echo "${1}") <(echo "${2}") | ${HEAD} -n1`
  if [ "${RES}" = "${1}" ]; then
    return 0
  fi
  return 1
}

parse_args() {
  while [ ! -z "${1}" ]; do
    case ${1} in
      -d|--debug)
        LOG_DEBUG=1
        STDOUT=/dev/stdout
        STDERR=/dev/stderr
        log_debug "Running with debug output"
        ;;
      -y|--yes)
        ASSUME_YES=1
        ;;
      --no-welcome)
        NO_WELCOME=1
        ;;
      --no-banner)
        NO_BANNER=1
        ;;
      --dev)
        BASE_URL=dl-dev.flecs.tech
        ;;
      --root-dir)
        ROOT_DIR=${2}
        if [ -z "${ROOT_DIR}" ]; then
          log_error "argument --root-dir requires a value"
          log_error -q
          print_usage
          exit 1
        fi
        ;;
      --help)
        print_usage
        exit 0
        ;;
    esac
    shift
  done
}

welcome() {
  if [ -z "${NO_WELCOME}" ]; then
    # print welcome message and wait for confirmation, if not unattended
    log_info -n "${ME} is about to install FLECS for ${ARCH} on"
    if [ ! -z "${NAME}" ]; then
      log_info -n -q " ${NAME}"
      [ ! -z "${OS_VERSION}" ] && log_info -n -q " ${OS_VERSION}"
      [ ! -z "${CODENAME}" ] && log_info -n -q " (${CODENAME})"
      log_info -q
    else
      log_info -q " your device"
    fi
    confirm "Press enter to begin installation or Ctrl-C to cancel."
  fi
}

# tries to detect presence of a program by running its "--help" function
# and using `which` (not available on all platforms) as fallback
have_program() {
  if ${1} --help >/dev/null 2>&1; then
    echo ${1}
  else
    which ${1} 2>/dev/null
  fi
}
# wrapper around have_program that declares a global variable named like the
# program in uppercase (e.g. CURL=... for curl)
have() {
  log_debug -n "Looking for ${1}..."
  local TOOL=${1^^}
  local TOOL=${TOOL//-/_}
  local TOOL=${TOOL//./_}
  if [ -z "${!TOOL}" ]; then
    declare -g ${TOOL}=`have_program ${1}`
  fi
  [ ! -z "${!TOOL}" ] && log_debug -q " found" || log_debug -q " not found"
}

# wrapper for apt-get update
apt_update() {
  log_debug "apt-get update"
  if [ -z "${APT_GET}" ] || ! ${APT_GET} update 1>${STDOUT} 2>${STDERR}; then
    return 1
  fi
  return 0
}
# wrapper for apt-get install
apt_install() {
  log_debug "apt-get install $@"
  if [ -z "${APT_GET}" ] || ! ${APT_GET} -y install --reinstall $@ 1>${STDOUT} 2>${STDERR}; then
    return 1
  fi
  return 0
}
# wrapper for pacman -Syu
pacman_update() {
  if [ -z "${PACMAN}" ] || ! ${PACMAN} -Syu --noconfirm 1>${STDOUT} 2>${STDERR}; then
    return 1
  fi
  return 0
}
# wrapper for pacman -S
pacman_install() {
  if [ -z "${PACMAN}" ] || ! ${PACMAN} -S --needed --noconfirm $@ 1>${STDOUT} 2>${STDERR}; then
    return 1
  fi
  return 0
}
# wrapper for yum update
yum_update() {
  if [ -z "${YUM}" ] || ! ${YUM} update --assumeno 1>${STDOUT} 2>${STDERR}; then
    return 1
  fi
  return 0
}
# wrapper for yum install
yum_install() {
  if [ -z "${YUM}" ] || ! ${YUM} install --assumeyes 1>${STDOUT} 2>${STDERR}; then
    return 1
  fi
  return 0
}

can_install_program() {
  if [ -z "${OS_LIKE}" ]; then
    if [ ! -z "${APT_GET}" ] || [ ! -z "${PACMAN}" ] || [ ! -z "${YUM}" ]; then
      return 0
    fi
  else
    case ${OS_LIKE} in
      debian|fedora|arch)
        return 0
        ;;
    esac
  fi
  return 1
}
install_program() {
  # Before OS detection, we need to guess how to install programs
  if [ -z "${OS_LIKE}" ]; then
    if apt_update && apt_install $@; then
      return 0;
    elif pacman_update && pacman_install $@; then
      return 0
    elif yum_update && yum_install $@; then
      return 0
    fi
  # Afterwards we know how to do it
  else
    case ${OS_LIKE} in
      debian)
        if apt_update && apt_install $@; then
          return 0
        fi
        ;;
      fedora)
        if yum_update && yum_install $@; then
          return 0
        fi
        ;;
      arch)
        if yum_update && yum_install $@; then
          return 0
        fi
        ;;
    esac
  fi
  return 1
}

# detect which tools are available on the system
detect_tools() {
  log_debug "Checking availability of required tools..."
  TOOLS=(apt-get apt-key curl docker docker-compose docker-init dpkg grep gpg head ldconfig mktemp opkg pacman rpm sed sort tar systemctl uname update-rc.d wget yum)
  for TOOL in ${TOOLS[@]}; do
    have ${TOOL}
  done
}
verify_tool_alternatives() {
  TOOL_ALTERNATIVES=("$@")
  for TOOL in "${TOOL_ALTERNATIVES[@]}"; do
    REQ=${TOOL^^}
    REQ=${REQ//-/_}
    if [ ! -z "${!REQ}" ]; then
      local FOUND="true"
      log_debug "Found alternative ${TOOL} for ${TOOL_ALTERNATIVES[@]}"
    fi
  done
  if [ -z "${FOUND}" ]; then
    return 1
  fi
  return 0
}
# quit if required tools are missing
verify_tools() {
  INSTALL_PACKAGES=""
  log_debug "Verifying presence of required basic tools..."
  REQUIRED_TOOLS=(head sort)
  for TOOL in ${REQUIRED_TOOLS[@]}; do
    REQ=${TOOL^^}
    REQ=${REQ//-/_}
    if [ -z "${!REQ}" ]; then
      INSTALL_PACKAGES="${INSTALL_PACKAGES} ${TOOL}"
    fi
  done
  local ALTERNATIVES=("apt-get;dpkg;ipkg;opkg;rpm;tar" "sed;grep" "curl;wget" "uname;ldconfig")
  for ALTERNATIVE in "${ALTERNATIVES[@]}"; do
    IFS=";" read -r -a TOOL_ALTERNATIVE <<< "${ALTERNATIVE}"
    if ! verify_tool_alternatives "${TOOL_ALTERNATIVE[@]}"; then
      INSTALL_PACKAGES="${INSTALL_PACKAGES} ${TOOL_ALTERNATIVE[0]}"
    fi
  done
}

# check internet connection in multiple ways
check_connectivity() {
  log_info -n "Checking internet connectivity..."
  if [ ! -z "${CURL}" ]; then
    if ${CURL} ${BASE_URL} 1>${STDOUT} 2>${STDERR}; then
      echo "OK"
      return 0
    fi
 elif [ ! -z "${WGET}" ]; then
    if ${WGET} -q ${BASE_URL} 1>${STDOUT} 2>${STDERR}; then
      echo "OK"
      return 0
    fi
  fi
  log_info -q "failed"
  log_fatal "Please make sure your device is online before running ${ME}"
  return 1;
}

machine_to_arch() {
  case ${MACHINE} in
    amd64|x86_64|x86-64)
      ARCH="amd64"
      ;;
    arm64|aarch64)
      ARCH="arm64"
      ;;
    armhf|armv7l)
      ARCH="armhf"
      ;;
    *)
      ARCH="unknown"
      ;;
  esac
}
detect_arch() {
  log_debug -n "Detecting system architecture..."
  if [ ! -z "${DPKG}" ]; then
    MACHINE=`${DPKG} --print-architecture`
  elif [ ! -z "${UNAME}" ]; then
    MACHINE=`${UNAME} -m` 
  elif [ ! -z "${LDCONFIG}" ] && [ ! -z "${GREP}" ]; then
    MACHINE=`${LDCONFIG} -p | ${GREP} -oP "(?<=\/ld-linux-)[^.]+"`
  fi
  machine_to_arch
  if [ "${ARCH}" = "unknown" ]; then
    log_debug -q " failed"
    internal_error "Architecture for ${MACHINE} is unsupported"
  fi
  log_debug -q " ${ARCH}"
}

parse_os_release() {
  if [ ! -z "${SED}" ]; then
    ${SED} -nE "s/^${1}=\"?([^\"]+)\"?$/\1/p" /etc/os-release 2>/dev/null
  elif [ ! -z "${GREP}" ]; then
    if ! grep -oP "(?<=^${1}=\").+(?=\")" /etc/os-release 2>/dev/null; then
      grep -oP "(?<=^${1}=).+$" /etc/os-release 2>/dev/null
    fi
  fi
}
detect_os() {
  log_debug "Detecting operating system..."
  OS=`parse_os_release "ID"`
  log_debug "Detected OS ${OS}"

  case ${OS} in
    debian|raspbian|ubuntu)
      OS_VERSION=`parse_os_release "VERSION_ID"`
      CODENAME=`parse_os_release "VERSION_CODENAME"`
      OS_LIKE="debian"
      ;;
    fedora|rhel)
      OS_VERSION=`parse_os_release "VERSION_ID"`
      OS_LIKE="fedora"
      log_warning "Fedora-based distributions that use podman are not yet supported. Please ensure"
      log_warning "you have Docker installed instead of podman, or follow the instructions found at"
      log_warning "https://docs.docker.com/engine/install/fedora/ to install Docker."
      log_warning "If you cannot use Docker for some reason, please contact us at info@flecs.tech"
      log_warning "for further information about podman support."
      confirm_yn "Continue"
      ;;
    arch)
      OS_LIKE=arch
      ;;
    *)
      OS_LIKE=other
      ;;
  esac
  NAME=`parse_os_release "NAME"`
  log_debug "Detected OS_VERSION ${OS_VERSION}"
  log_debug "Detected CODENAME ${CODENAME}"
  log_debug "Detected NAME ${NAME}"

  detect_arch
}

DEBIAN_VERSIONS=(10 11 12)
DEBIAN_CODENAMES=(buster bullseye bookworm)

UBUNTU_VERSIONS=(20.04 22.04 22.10 23.04)
UBUNTU_CODENAMES=(focal jammy kinetic lunar)

RHEL_VERSIONS=(8.8 9.2)
FEDORA_VERSIONS=(37 38)

verify_os_version() {
  if [ -z "${OS_VERSION}" ]; then
    internal_error "OS_VERSION not set in verify_os_version"
  fi

  for i in "${!VERIFY_VERSIONS[@]}"; do
    if [[ "${OS_VERSION}" == "${VERIFY_VERSIONS[$i]}" ]]; then
      local SUPPORTED="true"
      break
    fi
  done

  if [[ "${SUPPORTED}" != "true" ]]; then
    if cmp_less "${VERIFY_VERSIONS[-1]}" "${OS_VERSION}"; then
      local NEWER="true"
    fi
  fi

  if [[ "${SUPPORTED}" != "true" ]]; then
    if [[ "${NEWER}" != "true" ]]; then
      if [ ! -z "${CODENAME}" ]; then
        log_error "You are running an outdated version ${OS_VERSION} (${CODENAME}) of your OS. Supported versions are"
      else
        log_error "You are running an outdated version ${OS_VERSION} of your OS. Supported versions are"
      fi
      for i in "${!VERIFY_VERSIONS[@]}"; do
        if [ ! -z "${VERIFY_CODENAMES[$i]}" ]; then
          log_error "    ${VERIFY_VERSIONS[$i]} (${VERIFY_CODENAMES[$i]})"
        else
          log_error "    ${VERIFY_VERSIONS[$i]}"
        fi
      done
      log_fatal
    else
      log_warning "You are running an unsupported version of your OS. Supported versions are"
      for i in "${!VERIFY_VERSIONS[@]}"; do
        if [ ! -z "${VERIFY_CODENAMES[$i]}" ]; then
          log_warning "    ${VERIFY_VERSIONS[$i]} (${VERIFY_CODENAMES[$i]})"
        else
          log_warning "    ${VERIFY_VERSIONS[$i]}"
        fi
      done
      if [ ! -z "${CODENAME}" ]; then
        log_warning "Your version ${OS_VERSION} (${CODENAME}) seems more recent, so continuing anyway"
      else
        log_warning "Your version ${OS_VERSION} seems more recent, so continuing anyway"
      fi
    fi
  fi
}

verify_os() {
  case ${OS} in
    debian|raspbian)
      VERIFY_VERSIONS=("${DEBIAN_VERSIONS[@]}")
      VERIFY_CODENAMES=("${DEBIAN_CODENAMES[@]}")
      verify_os_version
      ;;
    ubuntu|pop)
      VERIFY_VERSIONS=("${UBUNTU_VERSIONS[@]}")
      VERIFY_CODENAMES=("${UBUNTU_CODENAMES[@]}")
      verify_os_version
      ;;
    fedora)
      VERIFY_VERSIONS=("${FEDORA_VERSIONS[@]}")
      VERIFY_CODENAMES=
      verify_os_version
      ;;
    rhel)
      VERIFY_VERSIONS=("${RHEL_VERSIONS[@]}")
      VERIFY_CODENAMES=
      verify_os_version
      ;;
    arch)
      # rolling release, so no version to check
      ;;
    *)
      EXPERIMENTAL=true
  esac
}

determine_docker_version() {
  log_info -n "Determining Docker version..."
  if [ -z "${DOCKER}" ]; then
    echo " none"
    log_fatal "Docker is not installed on your device"
  fi

  if ${DOCKER} -v 2>/dev/null | ${GREP} podman >/dev/null 2>&1; then
    DOCKER_NAME="podman"
  else
    DOCKER_NAME="Docker"
  fi

  TIMEOUT=5
  while ! ${DOCKER} version >/dev/null 2>&1 && [ ${TIMEOUT} -ge 1 ]; do
    sleep 1
    TIMEOUT=$((TIMEOUT-1))
  done
  if [ ! -z "${SED}" ]; then
    DOCKER_CLIENT_VERSION=`${DOCKER} -v 2>/dev/null | ${SED} -nE 's/^[^0-9]+([0-9\.]+).*$/\1/p'`
  elif [ ! -z "${GREP}" ]; then
    DOCKER_CLIENT_VERSION=`${DOCKER} -v 2>/dev/null | ${GREP} -oP "([0-9]+[\.]){2}[0-9]+" | ${HEAD} -n1`
  fi

  echo " found ${DOCKER_NAME}"

  DOCKER_API_VERSION="unknown"
  if ! ${DOCKER} version >/dev/null 2>&1; then
    log_warning "Could not determine Docker API version. Maybe you need to start it using"
    log_warning "    'systemctl enable --now docker.service' or"
    log_warning "    '/etc/init.d/docker start'"
  else
    DOCKER_API_VERSION=`${DOCKER} version --format '{{.Server.APIVersion}}' 2>/dev/null`
  fi

  if [ -z "${DOCKER_API_VERSION}" ] || [ -z "${DOCKER_CLIENT_VERSION}" ]; then
    internal_error "Could not determine Docker version."
    return 1
  fi
  log_info "    Client: ${DOCKER_CLIENT_VERSION}"
  log_info "    API: ${DOCKER_API_VERSION}"

  return 0
}

# verifies that a supported Docker version is installed and running. Podman is detected as such,
# and will currently be rejected as support is in development.
DOCKER_OK=0
DOCKER_OUTDATED=2
verify_docker_version() {
  if [ "${DOCKER_NAME}" = "podman" ]; then
    MIN_DOCKER_API_VERSION="4.5.0"
    MIN_DOCKER_CLIENT_VERSION="4.5.0"
    log_error "Podman is currently unsupported."
    log_fatal "Please contact us at info@flecs.tech if you require podman support"
  else
    MIN_DOCKER_API_VERSION="1.41"
    MIN_DOCKER_CLIENT_VERSION="20.10.5"
  fi

  if cmp_less "${DOCKER_CLIENT_VERSION}" "${MIN_DOCKER_CLIENT_VERSION}"; then
    log_error "FLECS requires at least ${DOCKER_NAME} client version ${MIN_DOCKER_CLIENT_VERSION}"
    log_error "The available client version is ${DOCKER_CLIENT_VERSION}"
    return ${DOCKER_OUTDATED}
  fi

  if cmp_less "${DOCKER_API_VERSION}" "${MIN_DOCKER_API_VERSION}" && [ ! "${DOCKER_API_VERSION}" = "unknown" ]; then
    log_error "FLECS requires at least ${DOCKER_NAME} API version ${MIN_DOCKER_API_VERSION}."
    log_error "The available API version is ${DOCKER_API_VERSION}"
    return ${DOCKER_OUTDATED}
  fi

  return ${DOCKER_OK}
}

# creates a new file in /etc/apt/sources.list.d
create_apt_list() {
  echo ${2} >/etc/apt/sources.list.d/${1}.list
  return $?
}

# removes a file from /etc/apt/sources.list.d
remove_apt_list() {
  rm /etc/apt/sources.list.d/${1}.list
  return $?
}

# Trust Debian keys on Raspbian. This is required to update libseccomp2 from
# buster-backports on Raspbian versions before bullseye.
add_debian_keys() {
  if [ -z "${GPG}" ]; then
    log_info "Installing prerequisite gpg"
    if ! apt_update || ! apt_install gpg; then
      log_fatal "Could not install prerequisite gpg"
    fi
    have gpg
  fi
  if [ -z "${APT_KEY}" ]; then
    internal_error "add_debian_keys called on incompatible platform"
  fi
  log_info "Adding Debian keys to trusted apt keys"
  local KEYSERVER="keyserver.ubuntu.com"
  local KEYS=(04EE7237B7D453EC 648ACFD622F3D138)
  for KEY in "${KEYS[@]}"; do
    if ! ${GPG} --keyserver ${KEYSERVER} --recv-keys ${KEY} 1>${STDOUT} 2>${STDERR}; then
      log_fatal "Failed to receive key ${KEY}"
    fi
    if ! ${GPG} --export ${KEY} | ${APT_KEY} add - 1>${STDOUT} 2>${STDERR}; then
      log_fatal "Failed to trust key ${KEY}"
    fi
  done
}

# For Debian versions before bullseye, some Docker containers will not run
# correctly due to an incompatible version of libseccomp2. This function will
# install a suitable version from the buster-backports repository.
# On Raspbian, we need to add Debian signing keys to the list of trusted apt
# keys first. On Debian we can proceed with the installation directly.
install_libseccomp2() {
  # verify preconditions before continuing
  if [[ "${OS}" != "debian" ]] && [[ "${OS}" != "raspbian" ]]; then
    internal_error "install_libseccomp2 called on incompatible platform (${OS})"
  fi
  if  [[ "${CODENAME}" != "buster" ]]; then
    internal_error "install_libseccomp2 called on incompatible platform (${CODENAME})" 1>&2
  fi

  log_info "Installing prerequisite libseccomp2"
  # trust Debian keys in case of Raspbian
  if [ "${OS}" == "raspbian" ]; then
    add_debian_keys
  fi

  # create apt list
  if ! create_apt_list flecs_buster-backports "deb http://deb.debian.org/debian buster-backports main"; then
    remove_apt_list flecs_buster-backports
    log_fatal "Could not create buster-backports.list"
  fi
  # update package lists
  if ! apt_update; then
    remove_apt_list flecs_buster-backports
    log_fatal "apt-get update returned error in install_libseccomp2"
  fi
  # install libseccomp2
  if ! apt_install libseccomp2/buster-backports; then
    remove_apt_list flecs_buster-backports
    apt_update
    log_fatal "apt-get install returned error in install_libseccomp2"
  fi
}

add_docker_sources() {
  if [ -z "${GPG}" ]; then
    log_info "Installing prerequisite gpg"
    if ! apt_update || ! apt_install gpg; then
      log_fatal "Could not install prerequisite gpg"
    fi
    have gpg
  fi
  if [ ! -z "${CURL}" ]; then
    ${CURL} -fsSL https://download.docker.com/linux/${OS}/gpg | \
      ${GPG} --batch --yes --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg
  elif [ ! -z "${WGET}" ]; then
    ${WGET} -q -O - https://download.docker.com/linux/${OS}/gpg | \
      ${GPG} --batch --yes --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg
  fi
  if [ ! -f "/usr/share/keyrings/docker-archive-keyring.gpg" ]; then
    log_fatal "Could not retrieve apt keys for Docker"
  fi

  if [ -z "${DPKG}" ]; then
    internal_error "add_docker_sources called in incompatible platform"
  fi
  if ! create_apt_list docker "deb [arch=$(${DPKG} --print-architecture) signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/${OS} ${CODENAME} stable"; then
    log_fatal "Could not create docker.list"
  fi
  apt_update
  apt_install docker-ce docker-ce-cli
}

install_docker_debian() {
  echo " apt-get"
  case ${1} in
    buster)
      log_warning "The Docker installation that is provided by your operating system is outdated"
      log_warning "and will not work with FLECS. Instead, official Docker packages provided by"
      log_warning "docker.com will be installed."
      local DOCKER_PACKAGE="docker-ce docker-ce-cli"
      install_libseccomp2
      add_docker_sources
      ;;
    *)
      local DOCKER_PACKAGE="docker.io"
  esac

  if ! apt_update; then
    log_fatal "apt_update returned error in install_docker"
  fi
  if ! apt_install ${DOCKER_PACKAGE}; then
    log_fatal "apt_install install returned error in install_docker"
  fi
}

install_docker_fedora() {
  echo " yum"
  if [ -z "${YUM}" ]; then
    internal_error "yum not present in install_docker_fedora"
  fi
  if ! yum_update; then
    log_fatal "yum_update returned error in install_docker"
  fi
  if ! yum_install podman-docker; then
    log_fatal "yum_install returned error in install_docker"
  fi
}

install_docker_arch() {
  echo " pacman"
  if [ -z "${PACMAN}" ]; then
    internal_error "pacman not present in install_docker_arch"
  fi
  if ! pacman_update; then
    log_fatal "pacman_update returned error in install_docker"
  fi
  if ! pacman_install docker; then
    log_fatal "pacman_install returned error in install_docker"
  fi
}

install_docker() {
  case ${OS_LIKE} in
    debian)
      log_info -n "Installing Docker using"
      install_docker_debian ${CODENAME}
      ;;
    fedora)
      log_warning "Automatic Docker installation on fedora is currently unsupported"
      #install_docker_fedora
      ;;
    arch)
      log_info -n "Installing Docker using"
      install_docker_arch
      ;;
    *)
      log_fatal "Docker not installed and cannot install automatically"
  esac
  log_info "Done installing Docker. Restarting..."
  exec "${SCRIPTNAME}" --no-banner --no-welcome ${ARGS}
}

start_docker() {
  if ! docker version >/dev/null 2>&1; then
    log_info -n "Attempting to start Docker..."
    if [ ! -z "${SYSTEMCTL}" ] && ${SYSTEMCTL} enable --now docker >/dev/null 2>&1; then
      echo " OK (systemctl)"
      return 0
    elif [ -f "/etc/init.d/docker " ] && /etc/init.d/docker start >/dev/null 2>&1; then
      echo " OK (init.d)"
      return 0
    elif [ ! -z "${DOCKER_INIT}" ] && (${DOCKER_INIT} -- dockerd --host=unix:///var/run/docker.sock >/dev/null 2>&1 &); then
      echo " OK (docker-init)"
      return 0
    else
      :;
      #if bash -c 'containerd >/dev/null 2>&1'; then
      #  if bash -c 'dockerd >/dev/null 2>&1'; then
      #    echo " OK (raw)"
      #    return 0
      #  fi
      #fi
    fi
    echo " failed"
    return 1
  fi
  return 0
}

determine_latest_version() {
  log_info -n "Determining latest FLECS version..."
  # try through curl first, if available
  if [ ! -z "${CURL}" ]; then
    VERSION_CORE=`${CURL} -fsSL ${BASE_URL}/flecs/latest_flecs_${ARCH}`
    VERSION_WEBAPP=`${CURL} -fsSL ${BASE_URL}/webapp/latest_flecs-webapp_${ARCH}`
  # use wget as fallback, if available
  elif [ ! -z "${WGET}" ]; then
    VERSION_CORE=`${WGET} -q -O - ${BASE_URL}/flecs/latest_flecs_${ARCH}`
    VERSION_WEBAPP=`${WGET} -q -O - ${BASE_URL}/webapp/latest_flecs-webapp_${ARCH}`
  fi
  if [ ! -z "${VERSION_CORE}" ] && [ ! -z "${VERSION_WEBAPP}" ]; then
    echo " OK"
    log_info "    Core: ${VERSION_CORE}"
    log_info "    WebApp: ${VERSION_WEBAPP}"
  else
    echo " failed"
    log_fatal "Could not determine latest version of FLECS"
  fi
}

guess_package_format() {
  case ${OS_LIKE} in
    debian)
      PKGFORMAT=deb
      ;;
    fedora)
      PKGFORMAT=rpm
      ;;
    arch)
      PKGFORMAT=tar
      ;;
    other)
      if [ ! -z "${APT_GET}" ] || [ ! -z "${DPKG}" ] || [ ! -z "${OPKG}" ]; then
        PKGFORMAT=deb
      elif [ ! -z "${RPM}" ]; then
        PKGFORMAT=rpm
      elif [ ! -z "${TAR}" ]; then
        PKGFORMAT=tar
      else
        log_fatal "Cannot find suitable package format in (deb|rpm|tar)"
      fi
      ;;
  esac
}

create_download_dir() {
  DOWNLOAD_DIR=`mktemp -d`
  if [ ! -d "${DOWNLOAD_DIR}" ]; then
    DOWNLOAD_DIR="/tmp/flecs-install-tmp"
    mkdir -p ${DOWNLOAD_DIR}
  fi
  if [ ! -d "${DOWNLOAD_DIR}" ]; then
    log_fatal "Could not create ${DOWNLOAD_DIR}"
  fi
}

download_flecs() {
  if ! cd ${DOWNLOAD_DIR}; then
    log_fatal "Could not cd to ${DOWNLOAD_DIR}"
  fi
  log_info "Downloading FLECS as ${PKGFORMAT}"

  PACKAGES=(flecs_${VERSION_CORE}_${ARCH}.${PKGFORMAT} flecs-webapp_${VERSION_WEBAPP}_${ARCH}.${PKGFORMAT})
  DIRS=(flecs webapp)
  VERSIONS=(${VERSION_CORE} ${VERSION_WEBAPP})
  for i in ${!PACKAGES[@]}; do
    if [ ! -z "${CURL}" ]; then
      if ! ${CURL} -fsSL --output - ${BASE_URL}/${DIRS[$i]}/${VERSIONS[$i]}/${PKGFORMAT}/${PACKAGES[$i]} >${PACKAGES[$i]}; then
        log_fatal "Could not download ${PACKAGES[$i]} through ${CURL}"
      fi
    elif [ ! -z "${WGET}" ]; then
      if ! ${WGET} -q ${BASE_URL}/${DIRS[$i]}/${VERSIONS[$i]}/${PKGFORMAT}/${PACKAGES[$i]}; then
        log_fatal "Could not download ${PACKAGES[$i]} through ${WGET}"
      fi
    fi
  done
  for PACKAGE in ${PACKAGES[@]}; do
    if [ ! -f "${PACKAGE}" ]; then
      internal_error "Package ${PACKAGE} missing after download"
    fi
  done
  return 0
}

install_flecs() {
  log_info -n "Installing FLECS using"
  case ${PKGFORMAT} in
    deb)
      if [ ! -z "${APT_GET}" ]; then
        log_info -q " apt-get"
        for PACKAGE in "${PACKAGES[@]}"; do
          if ! apt_install ${DOWNLOAD_DIR}/${PACKAGE}; then
            log_fatal "Could not install ${PACKAGE} through apt-get"
          fi
        done
      elif [ ! -z "${DPKG}" ]; then
        log_info -q " dpkg"
        for PACKAGE in "${PACKAGES[@]}"; do
          if ! ${DPKG} --install ${DOWNLOAD_DIR}/${PACKAGE} 1>${STDOUT} 2>${STDERR}; then
            log_fatal "Could not install ${PACKAGE} through dpkg"
          fi
        done
      elif [ ! -z "${OPKG}" ]; then
        log_info -q " opkg"
        for PACKAGE in "${PACKAGES[@]}"; do
          if ! ${OPKG} --install ${DOWNLOAD_DIR}/${PACKAGE} 1>${STDOUT} 2>${STDERR}; then
            log_fatal "Could not install ${PACKAGE} through opkg"
          fi
        done
      else
        internal_error "Neither apt-get nor dpkg/opkg available to install deb package"
      fi
      ;;
    rpm)
      log_info -q " rpm"
      log_fatal "rpm package format is currently unsupported"
      ;;
    tar)
      log_info -q " tar"
      for PACKAGE in "${PACKAGES[@]}"; do
        if [ ! -z "${SYSTEMCTL}" ]; then
          #systemd
          if ! ${TAR} -C ${ROOT_DIR} -xf ${PACKAGE} --exclude=etc; then
            log_fatal "Could not install ${PACKAGE} through tar"
          fi
        elif [ ! -z "${DOCKER_COMPOSE}" ]; then
          #docker-compose
          if ! ${TAR} -C ${ROOT_DIR} -xf ${PACKAGE} --exclude=usr --exclude=etc/init.d; then
            log_fatal "Could not install ${PACKAGE} through tar"
          fi
        else
          #init.d
          if ! ${TAR} -C ${ROOT_DIR} -xf ${PACKAGE} --exclude=usr; then
            log_fatal "Could not install ${PACKAGE} through tar"
          fi
        fi
      done
      ;;
  esac
  :;
}

enable_flecs() {
  if [ ! -z "${SYSTEMCTL}" ]; then
    ${SYSTEMCTL} is-enabled flecs >/dev/null 2>&1
    if [ $? -ne 0 ]; then
      if confirm_yn "FLECS is not enabled by default on your system. Enable and start FLECS now"; then
        ${SYSTEMCTL} enable --now flecs >/dev/null 2>&1
        ${SYSTEMCTL} enable --now flecs-webapp >/dev/null 2>&1
      else
        log_info "Use"
        log_info "  systemctl enable --now flecs"
        log_info "  systemctl enable --now flecs-webapp"
        log_info "to enable and start FLECS"
      fi
    fi
  elif [ ! -z "${UPDATE_RC_D}" ]; then
    if confirm_yn "FLECS is not enabled by default on your system. Enable and start FLECS now"; then
      ${UPDATE_RC_D} flecs defaults 81 81
      ${UPDATE_RC_D} flecs-webapp defaults 80 80
      /etc/init.d/flecs-webapp start
      /etc/init.d/flecs start
    else
      log_info "Use"
      log_info "  update-rc.d flecs defaults 81 81"
      log_info "  update-rc.d flecs-webapp defaults 80 80"
      log_info "  /etc/init.d/flecs-webapp start"
      log_info "  /etc/init.d/flecs start"
      log_info "to enable and start FLECS"
    fi
  elif [ ! -z "${DOCKER_COMPOSE}" ]; then
    if confirm_yn "FLECS is not enabled by default on your system. Enable and start FLECS now"; then
      ${DOCKER_COMPOSE} -f `readlink -f ${ROOT_DIR}/etc/opt/flecs/docker-compose.yml` up -d
    else
      log_info "Use"
      log_info "  docker-compose -f `readlink -f ${ROOT_DIR}/etc/opt/flecs/docker-compose.yml` up -d"
      log_info "to enable and start FLECS"
    fi
  fi
}

banner() {
  if [ -z "${NO_BANNER}" ]; then
    echo "                      ▒▒▒▒▒▒▒▒  ▒▒  ▒▒        ▒▒  ▒▒▒▒▒▒▒                       "
    echo "                      ▒▒        ▒▒  ▒▒            ▒▒    ▒▒                      "
    echo "                      ▒▒▒▒▒▒    ▒▒  ▒▒        ▒▒  ▒▒▒▒▒▒▒                       "
    echo "                      ▒▒        ▒▒  ▒▒        ▒▒  ▒▒                            "
    echo "                      ▒▒        ▒▒  ▒▒▒▒▒▒▒▒  ▒▒  ▒▒                            "
    echo "                      FLECS Installer for Linux Platforms                       "
    echo
    echo "                              https://flecs.tech/                               "
    echo
  fi
}

if [ -z "${FLECS_TESTING}" ]; then
  parse_args $*
  banner

  # ensure running as root
  if [ ${EUID} -ne 0 ]; then
    log_error "${ME} needs to run as root"
    if ! have sudo; then
      log_fatal "Please login as root user and restart installation"
    else
      if confirm_yn "Restart using sudo"; then
        exec ${SUDO} "${SCRIPTNAME}" --no-banner ${ARGS}
      else
        log_fatal "Cannot continue installation without root privileges"
      fi
    fi
  fi

  detect_tools
  verify_tools
  if [ ! -z "${INSTALL_PACKAGES}" ]; then
    log_info "${ME} requires the following packages to continue"
    log_info "    ${INSTALL_PACKAGES}"
    if ! can_install_program; then
      log_error "${ME} does not support automatic package installation on your device"
      log_fatal "Please install missing packages manually before running ${ME}"
    fi
    if confirm_yn "Automatically install these packages"; then
      if ! install_program ${INSTALL_PACKAGES}; then
        log_fatal "Could not install required dependencies"
      fi
    else
      log_fatal "Cannot continue without required dependencies"
    fi
    log_info "Done installing dependencies. Restarting..."
    exec "${SCRIPTNAME}" --no-banner --no-welcome ${ARGS}
  fi
  detect_os
  verify_os

  welcome

  # print warning for unsupported systems and wait for confirmation, if not unattended
  if [ "${EXPERIMENTAL}" == "true" ]; then
    log_warning "Your operating system is not officially supported by the installer."
    if [ ! -z "${OS}" ]; then
      if [ ! -z "${NAME}" ]; then
        log_warning "    Name: ${NAME} (${OS})"
      else
        log_warning "    OS: ${OS}"
      fi
    else
      if [ ! -z "${NAME}" ]; then
        log_warning "    Name: ${NAME}"
      fi
    fi
    [ ! -z "${OS_VERSION}" ] && log_warning "    Version: ${OS_VERSION}" || log_warning "    Version: unknown"
    log_warning

    log_warning "Installation might still succeed, depending on your exact system configuration."
    log_warning "No changes will be made to your system on failure, so it is usually safe to"
    log_warning "attempt installation anyway."
    confirm "Press enter to continue installation, or Ctrl-C to cancel."
    log_warning
  fi

  echo

  # make sure device is online
  check_connectivity

  # check if Docker is installed,
  if [ -z "${DOCKER}" ]; then
    install_docker
  fi
  start_docker
  determine_docker_version
  verify_docker_version
  if [ $? -eq ${DOCKER_OUTDATED} ]; then
    if [ "${CODENAME}" = "buster" ]; then
      if confirm_yn "Upgrade to docker-ce from official docker.com sources"; then
        install_docker
      else
        log_fatal "Please upgrade your Docker installation before installing FLECS"  
      fi
    else
      log_fatal "Please upgrade your Docker installation before installing FLECS"
    fi
  fi

  # query latest FLECS version online
  if ! determine_latest_version; then
    log_fatal "Could not determine latest version of FLECS"
  fi

  # create temporary directory and download FLECS
  create_download_dir
  guess_package_format
  download_flecs

  # perform installation
  install_flecs

  # enable service, if not automatic
  enable_flecs

  log_info "FLECS was successfully installed!"
fi
EOF

SCRIPTNAME=`readlink -f "${0}"`
if [ "${SCRIPTNAME}" != "/tmp/filip.sh" ]; then
  chmod +x /tmp/filip.sh
  if [ ! -t 0 ]; then
    exec /tmp/filip.sh $* </dev/tty
  else
    exec /tmp/filip.sh $*
  fi
fi
