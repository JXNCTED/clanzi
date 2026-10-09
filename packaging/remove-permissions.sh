#!/bin/sh
set -eu
if [ "$(id -u)" -ne 0 ]; then
  echo 'Run this remover with sudo.' >&2
  exit 1
fi
rm -f /etc/udev/rules.d/70-clanzi.rules /etc/modules-load.d/clanzi.conf
udevadm control --reload-rules
udevadm trigger --subsystem-match=misc --sysname-match=uinput
udevadm trigger --subsystem-match=hidraw
echo 'Only Olanzi permission rules were removed. Configuration was preserved.'
