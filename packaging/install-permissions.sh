#!/bin/sh
set -eu
if [ "$(id -u)" -ne 0 ]; then
  echo 'Run this installer with sudo. Do not run the Olanzi application as root.' >&2
  exit 1
fi
rules_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
install -m 0644 "$rules_dir/70-clanzi.rules" /etc/udev/rules.d/70-clanzi.rules
printf 'uinput\n' > /etc/modules-load.d/clanzi.conf
chmod 0644 /etc/modules-load.d/clanzi.conf
modprobe uinput
udevadm control --reload-rules
udevadm trigger --subsystem-match=misc --sysname-match=uinput
udevadm trigger --subsystem-match=hidraw
echo 'Permissions installed. Reconnect the AU05 receiver, then retry in Olanzi.'
