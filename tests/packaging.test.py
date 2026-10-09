"""Exercise maintainer hooks with temporary paths and fake kernel/udev commands.

This deliberately does not install rules or change permissions on the host.
"""

import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


PACKAGING = Path(__file__).resolve().parents[1] / "packaging"


class PermissionLifecycle(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="clanzi-packaging-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.bin = self.root / "bin"
        self.bin.mkdir()
        self.log = self.root / "commands"
        self.env = {**os.environ, "PATH": str(self.bin), "COMMAND_LOG": str(self.log)}
        for command in ("cmp", "rm"):
            (self.bin / command).symlink_to(shutil.which(command))
        for command in ("modprobe", "udevadm"):
            path = self.bin / command
            path.write_text('#!/bin/sh\nprintf "%s\\n" "' + command + ' $*" >> "$COMMAND_LOG"\nexit "${FAIL_COMMANDS:-0}"\n')
            path.chmod(0o755)
        (self.root / "run/udev").mkdir(parents=True)
        self.files = ("udev/rules.d/70-clanzi.rules", "modules-load.d/clanzi.conf")
        for file in self.files:
            vendor = self.root / "usr/lib" / file
            vendor.parent.mkdir(parents=True)
            shutil.copyfile(PACKAGING / Path(file).name, vendor)
            (self.root / "etc" / file).parent.mkdir(parents=True)
        for name in ("postinst", "postrm"):
            script = (PACKAGING / name).read_text()
            # Rewrite only absolute filesystem paths; execute the actual hook logic.
            for prefix in ("/etc/", "/usr/lib/", "/run/udev"):
                script = script.replace(prefix, str(self.root) + prefix)
            (self.root / name).write_text(script)

    def run_hook(self, name, action):
        result = subprocess.run(
            ["/bin/sh", str(self.root / name), action], env=self.env,
            capture_output=True, text=True, check=False,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        return result

    def commands(self):
        return self.log.read_text().splitlines() if self.log.exists() else []

    def test_configure_and_repeat(self):
        for _ in range(2):
            self.run_hook("postinst", "configure")
        expected = ["modprobe uinput", "udevadm control --reload-rules",
                    "udevadm trigger --subsystem-match=misc --sysname-match=uinput",
                    "udevadm trigger --subsystem-match=hidraw"]
        self.assertEqual(self.commands(), expected * 2)

    def test_migrates_unchanged_legacy_files(self):
        for file in self.files:
            shutil.copyfile(self.root / "usr/lib" / file, self.root / "etc" / file)
        self.run_hook("postinst", "configure")
        for file in self.files:
            self.assertFalse((self.root / "etc" / file).exists())
            self.assertTrue((self.root / "usr/lib" / file).exists())

    def test_preserves_local_overrides(self):
        rule = self.root / "etc" / self.files[0]
        rule.write_text("# local rule\n")
        module = self.root / "etc" / self.files[1]
        module.symlink_to(self.root / "usr/lib" / self.files[1])
        self.run_hook("postinst", "configure")
        self.assertEqual(rule.read_text(), "# local rule\n")
        self.assertTrue(module.is_symlink())

    def test_upgrade_and_abort_hooks_do_nothing(self):
        for action in ("upgrade", "failed-upgrade", "abort-install", "abort-upgrade", "disappear"):
            self.run_hook("postrm", action)
        self.run_hook("postinst", "abort-upgrade")
        self.assertEqual(self.commands(), [])
        for file in self.files:
            self.assertTrue((self.root / "usr/lib" / file).exists())

    def test_remove_and_purge_after_payload_removal(self):
        # dpkg removes owned payload files before postrm; hooks cannot depend on them.
        shutil.rmtree(self.root / "usr")
        config = self.root / "home/user/.config/olanzi-cross-platform/keymap.json"
        config.parent.mkdir(parents=True)
        config.write_text("saved mappings")
        override = self.root / "etc" / self.files[0]
        override.write_text("# local rule\n")
        for action in ("remove", "purge", "purge"):
            self.run_hook("postrm", action)
        self.assertEqual(len(self.commands()), 9)
        self.assertFalse(any("modprobe" in command for command in self.commands()))
        self.assertEqual(config.read_text(), "saved mappings")
        self.assertEqual(override.read_text(), "# local rule\n")

    def test_no_udev_daemon(self):
        (self.root / "run/udev").rmdir()
        self.run_hook("postinst", "configure")
        self.run_hook("postrm", "remove")
        self.assertEqual(self.commands(), ["modprobe uinput"])

    def test_removed_udev_dependency(self):
        (self.bin / "udevadm").unlink()
        self.run_hook("postrm", "purge")
        self.assertEqual(self.commands(), [])

    def test_activation_failures_warn_without_breaking_package_manager(self):
        self.env["FAIL_COMMANDS"] = "1"
        result = self.run_hook("postinst", "configure")
        self.assertIn("uinput could not be loaded", result.stderr)
        self.assertIn("permissions could not be refreshed", result.stderr)
        result = self.run_hook("postrm", "remove")
        self.assertIn("permissions could not be refreshed", result.stderr)


if __name__ == "__main__":
    unittest.main()
