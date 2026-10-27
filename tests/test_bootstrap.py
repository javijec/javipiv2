"""Checks offline del manifiesto y del bootstrap; no ejecutan instalaciones."""
from pathlib import Path
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[1]


def catalog():
    registry = {}
    for raw in (ROOT / "catalog/modules.tsv").read_text().splitlines():
        if not raw or raw.startswith("#"):
            continue
        module, group, deps, state, description = raw.split("\t")
        assert module not in registry
        registry[module] = set() if deps == "-" else set(deps.split(","))
        assert group and state in {"preview", "planned"} and description
    return registry


class BootstrapTests(unittest.TestCase):
    def test_catalog_has_21_modules(self):
        self.assertEqual(len(catalog()), 21)

    def test_profiles_are_complete_and_dependency_ordered(self):
        modules = catalog()
        for profile in ("minimal", "developer", "full"):
            rows = [row.strip() for row in (ROOT / f"profiles/{profile}.list").read_text().splitlines()
                    if row.strip() and not row.startswith("#")]
            self.assertEqual(rows[0], "core")
            self.assertEqual(len(rows), len(set(rows)))
            visited = set()
            for name in rows:
                self.assertIn(name, modules)
                self.assertLessEqual(modules[name], visited)
                visited.add(name)
            if profile == "full":
                self.assertEqual(set(rows), set(modules))

    def test_shell_scripts_have_no_write_actions_in_preview(self):
        result = subprocess.run(["bash", "-n", str(ROOT / "installer/bootstrap.sh")], capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        for profile in ("minimal", "developer", "full"):
            result = subprocess.run(["bash", str(ROOT / "installer/bootstrap.sh"), "--plan", profile],
                                    capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("no instala", result.stdout)
        result = subprocess.run(["bash", str(ROOT / "installer/bootstrap.sh"), "--apply"],
                                capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)


if __name__ == "__main__":
    unittest.main()
