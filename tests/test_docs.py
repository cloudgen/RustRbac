# TP-DOC-01 — user document structure. Primary law: requirement-python-readme.
# The badge matches the package string. Every PNG is an absolute https link.
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
README = (ROOT / "README.md").read_text(encoding="utf-8")
IMAGE_BASE = "https://raw.githubusercontent.com/cloudgen/PyRbac/main/screenshots/"
RELATED = (
    "https://github.com/cloudgen/PyRbac",
    "https://pypi.org/project/PyRbac/",
    "https://github.com/Wilgat/ChronicleLogger",
    "https://github.com/cloudgen/ciao",
    "https://github.com/cloudgen/ciao-lite",
    "https://github.com/cloudgen/safe-rm",
)
HEADINGS = (
    "Features",
    "Advantages",
    "Quick Installation",
    "Usage",
    "Screenshots",
    "Examples",
    "Platform Compatibility",
    "Related Projects",
    "Contributing",
    "License",
    "Last Update",
)
SHOTS = (
    "main-menu-en.png",
    "login-prompt.png",
    "access-menu.png",
    "setup-menu.png",
    "audit-menu.png",
    "system-log.png",
    "language-menu.png",
    "self-management.png",
    "main-menu-zh-hant.png",
    "main-menu-zh-hans.png",
    "main-menu-ja.png",
)


def _package_version():
    pyproject = (ROOT / "pyproject.toml").read_text(encoding="utf-8")
    init_text = (ROOT / "src" / "PyRbac" / "__init__.py").read_text(encoding="utf-8")
    py_match = re.search(r'(?m)^version = "([^"]+)"', pyproject)
    init_match = re.search(r'__version__ = "([^"]+)"', init_text)
    assert py_match and init_match
    assert py_match.group(1) == init_match.group(1)
    return py_match.group(1)


class TestReadme(unittest.TestCase):
    """TP-DOC-01."""

    def test_tp_doc_01_structure_badge_and_absolute_images(self):
        version = _package_version()
        self.assertEqual(version, "1.0.2")
        self.assertIn("version-1.0.2", README)
        for heading in HEADINGS:
            self.assertIn("\n## {0}\n".format(heading), "\n" + README)
        for url in RELATED:
            self.assertIn(url, README)
        for name in SHOTS:
            url = IMAGE_BASE + name
            self.assertIn(url, README)
            self.assertTrue((ROOT / "screenshots" / name).is_file(), name)
        self.assertNotIn("](screenshots/", README)
        self.assertIn("no session row", README)
        self.assertIn("asks for the account name", README)
        self.assertIn("creates the tables when they are missing", README)
        self.assertIn("admin123", README)
        self.assertIn("operator123", README)
        self.assertIn("clerk123", README)
        self.assertIn("lead123", README)
        self.assertIn("PBKDF2", README)
        self.assertIn("sys_session", README)
        self.assertIn("PYRBAC_ALLOW_REMOTE", README)
        self.assertNotIn("join-first.png", README)
        self.assertNotIn("vaults/", README)
