# RBAC, database, environment, web, and ChronicleLogger.
# Uses the pyrbac_test schema. Does not write ruoyi_db or ry.
import http.client
import os
import re
import sys
import tempfile
import unittest
import urllib.parse
from pathlib import Path

_SRC = Path(__file__).resolve().parents[1] / "src"
if str(_SRC) not in sys.path:
    sys.path.insert(0, str(_SRC))

from PyRbac.cli import Cli, main
from PyRbac.desk import Desk
from PyRbac.password_hash import PasswordHash
from PyRbac.schema import Schema
from PyRbac.settings import Settings, SettingsError


ROOT = Path(__file__).resolve().parents[1]
TABLES = (
    "sys_user_role",
    "sys_user_post",
    "sys_role_menu",
    "sys_role_dept",
    "sys_oper_log",
    "sys_logininfor",
    "sys_session",
    "sys_notice",
    "sys_dict_data",
    "sys_dict_type",
    "sys_config",
    "sys_user",
    "sys_role",
    "sys_menu",
    "sys_dept",
    "sys_post",
)


def _password_from_file():
    path = Path.home() / ".local" / "PyRbac" / "database"
    mode = path.stat().st_mode & 0o777
    if mode != 0o600:
        raise unittest.SkipTest("database file is not mode 0600")
    found = ""
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.startswith("PYRBAC_DB_PASSWORD="):
            found = line.split("=", 1)[1]
    if not found:
        raise unittest.SkipTest("database file has no password")
    return found


def _drop_tables(store):
    for name in TABLES:
        store.execute("DROP TABLE IF EXISTS {0}".format(name))


def _names(text):
    rows = []
    for line in text.splitlines()[1:]:
        if line == "(none)":
            return []
        rows.append(line.split("|")[1])
    return rows


class TestEnvironment(unittest.TestCase):
    """PYRBAC_* defaults, the 0600 file, and the loopback bind."""

    def test_password_file_mode_and_remote_refusal(self):
        home = tempfile.mkdtemp(prefix="pyrbac_env_", dir="/tmp")
        path = Path(home) / "database"
        path.write_text("PYRBAC_DB_PASSWORD=from-file\n", encoding="utf-8")
        os.chmod(path, 0o644)
        loose = {
            "PYRBAC_HOME": home,
            "HOME": home,
            "PYRBAC_DB_PASSWORD": "",
        }
        self.assertEqual(Settings(environ=loose).password(), "")
        os.chmod(path, 0o600)
        self.assertEqual(Settings(environ=loose).password(), "from-file")
        env_wins = dict(loose)
        env_wins["PYRBAC_DB_PASSWORD"] = "from-env"
        self.assertEqual(Settings(environ=env_wins).password(), "from-env")
        remote = {"PYRBAC_BIND": "0.0.0.0", "PYRBAC_PORT": "8765"}
        with self.assertRaises(SettingsError) as caught:
            Settings(environ=remote).bind()
        self.assertIn("Refusing to listen", str(caught.exception))
        remote["PYRBAC_ALLOW_REMOTE"] = "yes"
        self.assertEqual(Settings(environ=remote).bind()[0], "0.0.0.0")
        self.assertEqual(Settings(environ={}).bind(), ("127.0.0.1", 8765))


class TestPasswordHash(unittest.TestCase):
    def test_pbkdf2_round_trip(self):
        hasher = PasswordHash()
        stored = hasher.hash("admin123")
        self.assertTrue(stored.startswith("pbkdf2_sha256$"))
        self.assertTrue(hasher.verify("admin123", stored))
        self.assertFalse(hasher.verify("admin123", "not-this-scheme"))
        self.assertFalse(hasher.verify("other", stored))


class TestRbacDatabase(unittest.TestCase):
    """Seed, data scope, and permissions on pyrbac_test."""

    @classmethod
    def setUpClass(cls):
        cls.home = tempfile.mkdtemp(prefix="pyrbac_home_", dir="/tmp")
        cls.sqlite_file = os.path.join(cls.home, "pyrbac.sqlite3")
        cls._saved = {
            name: os.environ.get(name)
            for name in (
                "PYRBAC_HOME",
                "PYRBAC_DB_ENGINE",
                "PYRBAC_SQLITE_FILE",
                "PYRBAC_DB_NAME",
                "PYRBAC_DB_PASSWORD",
                "PYRBAC_BIND",
                "PYRBAC_PORT",
                "PYRBAC_ALLOW_REMOTE",
            )
        }
        os.environ["PYRBAC_HOME"] = cls.home
        os.environ["PYRBAC_DB_ENGINE"] = "sqlite3"
        os.environ["PYRBAC_SQLITE_FILE"] = cls.sqlite_file
        os.environ.pop("PYRBAC_DB_PASSWORD", None)
        os.environ["PYRBAC_BIND"] = "127.0.0.1"
        os.environ["PYRBAC_PORT"] = "0"
        os.environ.pop("PYRBAC_ALLOW_REMOTE", None)
        cls.desk = Desk()
        _drop_tables(cls.desk.store)
        cls.assert_seed = cls.desk.db_init()

    @classmethod
    def tearDownClass(cls):
        web = cls.desk.web()
        if web.httpd is not None:
            web.stop()
        for name, value in cls._saved.items():
            if value is None:
                os.environ.pop(name, None)
            else:
                os.environ[name] = value

    def setUp(self):
        self.desk.sessions.clear()

    def test_run_initializes_when_tables_are_missing(self):
        def run(argv):
            app = Cli()
            from io import StringIO
            from contextlib import redirect_stderr, redirect_stdout
            out = StringIO()
            err = StringIO()
            with redirect_stdout(out), redirect_stderr(err):
                code = app.run(argv)
            return code, out.getvalue(), err.getvalue()

        _drop_tables(self.desk.store)
        try:
            code, out, err = run(["user-list"])
            self.assertEqual(code, 1, err)
            self.assertIn("Not signed in", err)
            count = self.desk.store.query_one("SELECT COUNT(*) AS n FROM sys_user")
            self.assertEqual(int(count["n"]), 4)
            admins = self.desk.store.query_one(
                "SELECT COUNT(*) AS n FROM sys_user WHERE user_name = 'admin'"
            )
            self.assertEqual(int(admins["n"]), 1)
            _drop_tables(self.desk.store)
            note = self.desk.serve_background()
            self.assertIn("http://127.0.0.1:", note)
            count = self.desk.store.query_one("SELECT COUNT(*) AS n FROM sys_user")
            self.assertEqual(int(count["n"]), 4)
            code, out, err = run([
                "login", "--user-name", "admin", "--password", Schema.INITIAL_ADMIN_PASSWORD,
            ])
            self.assertEqual(code, 0, err)
            self.assertIn("Signed in as admin", out)
            code, out, err = run(["db-init"])
            self.assertEqual(code, 0, err)
            self.assertIn("present", out)
            self.assertNotIn("seeded", out)
        finally:
            web = self.desk.web()
            if web.httpd is not None:
                web.stop()
            self.desk.db_init()

    def test_seed_once_and_scopes(self):
        self.assertIn("pyrbac_test: seeded", self.assert_seed)
        again = self.desk.db_init()
        self.assertIn("present", again)
        def run(argv):
            app = Cli()
            from io import StringIO
            from contextlib import redirect_stderr, redirect_stdout
            out = StringIO()
            err = StringIO()
            with redirect_stdout(out), redirect_stderr(err):
                code = app.run(argv)
            return code, out.getvalue(), err.getvalue()

        code, out, err = run([
            "login", "--user-name", "admin", "--password", Schema.INITIAL_ADMIN_PASSWORD,
        ])
        self.assertEqual(code, 0, err)
        self.assertIn("Signed in as admin", out)
        code, out, err = run(["whoami"])
        self.assertIn("admin=yes", out)
        code, out, err = run(["user-list"])
        names = _names(out)
        self.assertEqual(names, ["admin", "operator", "clerk", "lead"])
        code, out, err = run([
            "login", "--user-name", "operator", "--password", Schema.INITIAL_OPERATOR_PASSWORD,
        ])
        self.assertEqual(code, 0, err)
        code, out, err = run(["user-list"])
        self.assertEqual(_names(out), ["operator"])
        code, out, err = run([
            "user-add", "--user-name", "extra", "--nick-name", "Extra",
            "--dept-id", "102", "--role-id", "2",
        ])
        self.assertEqual(code, 1)
        self.assertIn("Missing permission", err)
        code, out, err = run(["config-set", "--config-key", "sys.user.initPassword", "--config-value", "nope"])
        self.assertEqual(code, 1)
        self.assertIn("Missing permission", err)
        code, out, err = run([
            "login", "--user-name", "lead", "--password", Schema.INITIAL_LEAD_PASSWORD,
        ])
        self.assertEqual(code, 0, err)
        code, out, err = run(["user-list"])
        self.assertEqual(_names(out), ["admin", "clerk", "lead"])
        code, out, err = run([
            "login", "--user-name", "clerk", "--password", Schema.INITIAL_CLERK_PASSWORD,
        ])
        self.assertEqual(code, 0, err)
        code, out, err = run(["user-list"])
        self.assertEqual(_names(out), ["clerk"])
        self.desk.store.execute("UPDATE sys_role SET data_scope = '5' WHERE role_id = 2")
        try:
            code, out, err = run(["user-list"])
            self.assertEqual(code, 0, err)
            self.assertEqual(_names(out), [])
        finally:
            self.desk.store.execute("UPDATE sys_role SET data_scope = '3' WHERE role_id = 2")
        code, out, err = run([
            "login", "--user-name", "admin", "--password", Schema.INITIAL_ADMIN_PASSWORD,
        ])
        self.assertEqual(code, 0, err)
        code, out, err = run(["user-remove", "--user-id", "1"])
        self.assertEqual(code, 1)
        self.assertIn("cannot be removed", err)
        code, out, err = run(["user-edit", "--user-id", "1", "--status", "1"])
        self.assertEqual(code, 1)
        self.assertIn("stays enabled", err)
        code, out, err = run(["role-grant", "--role-id", "1", "--menu-ids", "1", "--data-scope", "3"])
        self.assertEqual(code, 1)
        self.assertIn("not edited", err)
        code, out, err = run(["logout"])
        self.assertEqual(code, 0, err)
        code, out, err = run([
            "login", "--user-name", "missing", "--password", "admin123",
        ])
        self.assertEqual(code, 1)
        self.assertIn("Unknown account or wrong password", err)
        code, out, err = run(["notice-list"])
        self.assertEqual(code, 1)
        self.assertIn("Not signed in", err)

    def test_web_login_csrf_and_scope(self):
        note = self.desk.serve_background()
        self.assertIn("http://127.0.0.1:", note)
        url = note.splitlines()[0].split(": ", 1)[1]
        admin = self._sign_in("admin", Schema.INITIAL_ADMIN_PASSWORD, url)
        page = self._open(url + "users", admin)
        self.assertEqual(page.status, 200)
        self.assertIn("Add a user", page.body)
        self.assertIn("clerk", page.body)
        csrf = re.search(r'name="csrf" value="([^"]+)"', page.body).group(1)
        denied = self._post(url + "users/add", admin, {"user_name": "x", "csrf": "nope"})
        self.assertEqual(denied.status, 400)
        self.assertIn("form token", denied.body)
        operator = self._sign_in("operator", Schema.INITIAL_OPERATOR_PASSWORD, url)
        scoped = self._open(url + "users", operator)
        self.assertEqual(scoped.status, 200)
        self.assertNotIn("Add a user", scoped.body)
        self.assertIn("operator", scoped.body)
        self.assertNotIn(">clerk<", scoped.body)
        self.assertNotIn(">lead<", scoped.body)
        blocked = self._open(url + "roles", operator)
        self.assertEqual(blocked.status, 403)
        self.assertIn("Missing permission", blocked.body)
        bare = self._open(url + "users", "")
        self.assertIn(bare.status, (302, 303))
        self.assertEqual(bare.headers.get("Location"), "/login")

    def _connection(self, url):
        parsed = urllib.parse.urlparse(url)
        return http.client.HTTPConnection(parsed.hostname, parsed.port, timeout=5)

    def _sign_in(self, user_name, password, url):
        body = urllib.parse.urlencode({"user_name": user_name, "password": password})
        connection = self._connection(url)
        connection.request(
            "POST",
            "/login",
            body,
            {"Content-Type": "application/x-www-form-urlencoded"},
        )
        response = connection.getresponse()
        raw = response.read()
        cookie = response.getheader("Set-Cookie") or ""
        connection.close()
        self.assertIn(response.status, (302, 303), raw.decode("utf-8", "replace"))
        self.assertIn("pyrbac_session=", cookie)
        token = cookie.split("pyrbac_session=", 1)[1].split(";", 1)[0]
        self.assertNotIn(password, cookie)
        return token

    def _open(self, url, token):
        parsed = urllib.parse.urlparse(url)
        connection = self._connection(url)
        headers = {}
        if token:
            headers["Cookie"] = "pyrbac_session={0}".format(token)
        connection.request("GET", parsed.path or "/", headers=headers)
        response = connection.getresponse()
        page = _Page(response.status, response.read().decode("utf-8"), response.headers)
        connection.close()
        return page

    def _post(self, url, token, fields):
        parsed = urllib.parse.urlparse(url)
        connection = self._connection(url)
        data = urllib.parse.urlencode(fields)
        connection.request(
            "POST",
            parsed.path or "/",
            data,
            {
                "Cookie": "pyrbac_session={0}".format(token),
                "Content-Type": "application/x-www-form-urlencoded",
            },
        )
        response = connection.getresponse()
        page = _Page(response.status, response.read().decode("utf-8"), response.headers)
        connection.close()
        return page


class _Page:
    def __init__(self, status, body, headers):
        self.status = status
        self.body = body
        self.headers = headers


class TestChronicle(unittest.TestCase):
    """def main constructs ChronicleLogger and does not record a password."""

    def test_construct_and_secret_stay_out_of_the_log(self):
        source = (ROOT / "src" / "PyRbac" / "cli.py").read_text(encoding="utf-8")
        self.assertIn("ChronicleLogger(", source)
        self.assertIn(
            'ChronicleLogger is required. Next step: python -m pip install "ChronicleLogger>=1.3.1"',
            source,
        )
        logdir = tempfile.mkdtemp(prefix="pyrbac_log_", dir="/tmp")
        home = tempfile.mkdtemp(prefix="pyrbac_loghome_", dir="/tmp")
        saved = {
            name: os.environ.get(name)
            for name in (
                "PYRBAC_HOME",
                "PYRBAC_DB_ENGINE",
                "PYRBAC_SQLITE_FILE",
                "PYRBAC_DB_NAME",
                "PYRBAC_DB_PASSWORD",
            )
        }
        os.environ["PYRBAC_HOME"] = home
        os.environ["PYRBAC_DB_ENGINE"] = "sqlite3"
        os.environ["PYRBAC_SQLITE_FILE"] = os.path.join(home, "pyrbac.sqlite3")
        os.environ.pop("PYRBAC_DB_PASSWORD", None)
        secret = "s3cret-value-not-for-logs"
        try:
            Desk().db_init()
            code = main(
                ["login", "--user-name", "admin", "--password", secret],
                logdir=logdir,
            )
            self.assertEqual(code, 1)
            blob = ""
            for path in Path(logdir).rglob("*"):
                if path.is_file():
                    blob += path.read_text(encoding="utf-8", errors="replace")
            self.assertNotIn(secret, blob)
            self.assertIn("ChronicleLogger", source)
        finally:
            for name, value in saved.items():
                if value is None:
                    os.environ.pop(name, None)
                else:
                    os.environ[name] = value


class TestEngineChoice(unittest.TestCase):
    """sqlite3 is the default file. mysql is the other choice. Every interface offers both."""

    def _home(self):
        return tempfile.mkdtemp(prefix="pyrbac_engine_", dir="/tmp")

    def _settings(self, home, **extra):
        environ = {"PYRBAC_HOME": home, "HOME": home}
        environ.update(extra)
        return Settings(environ=environ, home=home)

    def test_default_is_the_sqlite_file(self):
        home = self._home()
        settings = self._settings(home)
        self.assertEqual(settings.engine(), "sqlite3")
        self.assertTrue(settings.sqlite_path().endswith(os.path.join("PyRbac", "pyrbac.sqlite3")) or settings.sqlite_path().endswith("pyrbac.sqlite3"))
        self.assertIn("pyrbac.sqlite3", settings.sqlite_path())
        lines = settings.choice_lines()
        self.assertIn("1 sqlite3", lines[1])
        self.assertIn("pyrbac.sqlite3", lines[1])
        self.assertIn("(default file)", lines[1])
        self.assertIn("2 mysql", lines[2])
        sentence = settings.apply_choice("1")
        self.assertIn("sqlite3", sentence)
        path = Path(home) / "database"
        self.assertEqual(path.stat().st_mode & 0o777, 0o600)
        self.assertIn("PYRBAC_DB_ENGINE=sqlite3", path.read_text(encoding="utf-8"))
        self.assertFalse((Path(home) / "pyrbac.sqlite3").exists())

    def test_mysql_choice_and_environment_wins(self):
        home = self._home()
        settings = self._settings(home)
        sentence = settings.apply_choice("mysql")
        self.assertIn("mysql", sentence)
        self.assertEqual(settings.engine(), "mysql")
        locked = self._settings(home, PYRBAC_DB_ENGINE="sqlite3")
        with self.assertRaises(SettingsError) as caught:
            locked.apply_choice("2")
        self.assertIn("environment", str(caught.exception))
        self.assertEqual(locked.apply_choice(""), locked.engine_sentence())
        with self.assertRaises(SettingsError):
            settings.apply_choice("postgres")

    def test_cli_prints_both_choices_without_opening_sqlite(self):
        home = self._home()
        saved = {
            name: os.environ.get(name)
            for name in ("PYRBAC_HOME", "PYRBAC_DB_ENGINE", "PYRBAC_SQLITE_FILE", "HOME")
        }
        os.environ["PYRBAC_HOME"] = home
        os.environ["HOME"] = home
        os.environ.pop("PYRBAC_DB_ENGINE", None)
        os.environ.pop("PYRBAC_SQLITE_FILE", None)
        try:
            from io import StringIO
            from contextlib import redirect_stderr, redirect_stdout
            app = Cli()
            out = StringIO()
            err = StringIO()
            with redirect_stdout(out), redirect_stderr(err):
                code = app.run(["db-engine"])
            self.assertEqual(code, 0, err.getvalue())
            text = out.getvalue()
            self.assertIn("1 sqlite3", text)
            self.assertIn("pyrbac.sqlite3", text)
            self.assertIn("2 mysql", text)
            self.assertFalse((Path(home) / "pyrbac.sqlite3").exists())
            out = StringIO()
            err = StringIO()
            with redirect_stdout(out), redirect_stderr(err):
                code = app.run(["db-engine", "--engine", "mysql"])
            self.assertEqual(code, 0, err.getvalue())
            self.assertIn("mysql", out.getvalue())
            stored = (Path(home) / "database").read_text(encoding="utf-8")
            self.assertIn("PYRBAC_DB_ENGINE=mysql", stored)
            self.assertFalse((Path(home) / "pyrbac.sqlite3").exists())
            err = StringIO()
            with redirect_stdout(StringIO()), redirect_stderr(err):
                code = app.run(["db-engine", "--engine", "oracle"])
            self.assertEqual(code, 1)
            self.assertIn("ERROR:", err.getvalue())
            self.assertIn("Next:", err.getvalue())
        finally:
            for name, value in saved.items():
                if value is None:
                    os.environ.pop(name, None)
                else:
                    os.environ[name] = value

    def test_sqlite_seed_file_mode_and_web_choices(self):
        home = self._home()
        data = os.path.join(home, "pyrbac.sqlite3")
        desk = Desk(environ={
            "PYRBAC_HOME": home,
            "HOME": home,
            "PYRBAC_DB_ENGINE": "sqlite3",
            "PYRBAC_SQLITE_FILE": data,
            "PYRBAC_BIND": "127.0.0.1",
            "PYRBAC_PORT": "0",
        })
        first = desk.db_init()
        self.assertIn("seeded", first)
        self.assertIn(data, first)
        second = desk.db_init()
        self.assertIn("present", second)
        self.assertEqual(os.stat(data).st_mode & 0o777, 0o600)
        page = desk.web().login_page("")
        self.assertIn("sqlite3", page)
        self.assertIn("pyrbac.sqlite3", page)
        self.assertIn("mysql", page)
        self.assertIn('name="engine"', page)
        plain = Desk(environ={
            "PYRBAC_HOME": home,
            "HOME": home,
            "PYRBAC_BIND": "127.0.0.1",
            "PYRBAC_PORT": "0",
        })
        plain_page = plain.web().login_page("")
        self.assertIn("(default file)", plain_page)
        self.assertIn("pyrbac.sqlite3", plain_page)
        signed = desk.access.login(
            "admin",
            Schema.INITIAL_ADMIN_PASSWORD,
            ipaddr="127.0.0.1",
            surface="web",
            user_agent="test",
        )
        body = desk.web().database_page(signed, "")
        self.assertIn("sqlite3", body)
        self.assertIn("mysql", body)
        self.assertIn("csrf", body)

        class Remote:
            client_address = ("203.0.113.9", 9)

        with self.assertRaises(SettingsError):
            desk.web()._apply_posted_engine(Remote(), {"engine": ["mysql"]})

        class Local:
            client_address = ("127.0.0.1", 9)

        open_choice = Desk(environ={"PYRBAC_HOME": home, "HOME": home})
        open_choice.web()._apply_posted_engine(Local(), {"engine": ["mysql"]})
        self.assertEqual(open_choice.settings.engine(), "mysql")
        self.assertIn(
            "PYRBAC_DB_ENGINE=mysql",
            (Path(home) / "database").read_text(encoding="utf-8"),
        )

    def test_menu_row_offers_the_engine(self):
        from PyRbac.menu_painter import MenuPainter
        rows = MenuPainter().rows_for("setup")
        self.assertIn((78, "db-engine", "sqlite3 file or mysql", "db-engine"), rows)
        self.assertIn("db-engine", Cli.PRODUCT_VERBS)
        self.assertNotIn("db-engine", Cli.DOMAIN_VERBS)

    def test_mysql_seed_when_the_server_is_reachable(self):
        try:
            password = _password_from_file()
        except unittest.SkipTest:
            raise
        home = self._home()
        desk = Desk(environ={
            "PYRBAC_HOME": home,
            "HOME": home,
            "PYRBAC_DB_ENGINE": "mysql",
            "PYRBAC_DB_NAME": "pyrbac_test",
            "PYRBAC_DB_USER": "pyrbac",
            "PYRBAC_DB_HOST": "127.0.0.1",
            "PYRBAC_DB_PORT": "3306",
            "PYRBAC_DB_PASSWORD": password,
        })
        try:
            desk.store.query_one("SELECT 1 AS n")
        except Exception:
            raise unittest.SkipTest("mysql is not reachable")
        _drop_tables(desk.store)
        first = desk.db_init()
        self.assertIn("seeded", first)
        self.assertIn("pyrbac_test", first)
        second = desk.db_init()
        self.assertIn("present", second)
        self.assertNotIn("ruoyi_db", first)
        self.assertNotIn("ry", first)
