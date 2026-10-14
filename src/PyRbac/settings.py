# =============================================================================
# Process settings for PyRbac.
# requirement-environment — names, defaults, and the mode-0600 password file.
# requirement-python-oop — class Settings and SettingsError.
# =============================================================================
from __future__ import annotations

import os


class SettingsError(Exception):
    """A missing or refused setting. The next step is for the operator."""

    def __init__(self, message, nxt, logger=None):
        super().__init__(message)
        self.logger = logger
        self.nxt = nxt
        if logger is not None:
            logger.log_message("instantiated", component="SettingsError")


class Settings:
    """Environment values for the database, the web bind, and the home folder.

    The environment wins. A mode-0600 file supplies a key only when the
    environment value is missing. Defaults never include a password.
    """

    APP_DIR = "PyRbac"
    FILE_NAME = "database"
    SQLITE_NAME = "pyrbac.sqlite3"
    ENGINES = ("sqlite3", "mysql")
    DEFAULTS = {
        "PYRBAC_DB_HOST": "127.0.0.1",
        "PYRBAC_DB_PORT": "3306",
        "PYRBAC_DB_USER": "pyrbac",
        "PYRBAC_DB_NAME": "pyrbac",
        "PYRBAC_BIND": "127.0.0.1",
        "PYRBAC_PORT": "8765",
    }
    LOOPBACK = ("127.0.0.1", "localhost", "::1")

    def __init__(self, environ=None, home=None, logger=None):
        self.logger = logger
        if logger is not None:
            logger.log_message("instantiated", component="Settings")
        if environ is None:
            environ = os.environ
        self.environ = environ
        if home is None:
            home = self.environ.get("HOME", "")
        self.home = home
        self._file_cache = None

    def app_home(self) -> str:
        """General Purpose: The folder that holds the language leaf, the session, and the database file."""
        override = self.environ.get("PYRBAC_HOME", "")
        if override:
            return override
        if not self.home:
            return ""
        return os.path.join(self.home, ".local", self.APP_DIR)

    def database_file(self) -> str:
        folder = self.app_home()
        if not folder:
            return ""
        return os.path.join(folder, self.FILE_NAME)

    def _file_map(self) -> dict:
        if self._file_cache is not None:
            return self._file_cache
        found = {}
        path = self.database_file()
        if path and os.path.isfile(path):
            mode = os.stat(path).st_mode & 0o777
            if mode != 0o600:
                self._file_cache = {}
                return self._file_cache
            try:
                with open(path, "r", encoding="utf-8") as handle:
                    lines = handle.read().splitlines()
            except OSError:
                lines = []
            for line in lines:
                if not line or line.startswith("#") or "=" not in line:
                    continue
                key, value = line.split("=", 1)
                found[key.strip()] = value
        self._file_cache = found
        return found

    def get(self, name: str) -> str:
        """General Purpose: One setting. Environment, then the 0600 file, then a default."""
        current = self.environ.get(name, "")
        if current != "":
            return current
        filed = self._file_map().get(name, "")
        if filed != "":
            return filed
        return self.DEFAULTS.get(name, "")

    def engine(self) -> str:
        """General Purpose: sqlite3 or mysql. An empty setting is the sqlite3 file."""
        value = self.get("PYRBAC_DB_ENGINE").strip().lower()
        if value == "":
            return "sqlite3"
        if value not in self.ENGINES:
            raise SettingsError(
                "PYRBAC_DB_ENGINE must be sqlite3 or mysql.",
                "unset PYRBAC_DB_ENGINE, or set sqlite3 or mysql",
                logger=self.logger,
            )
        return value

    def sqlite_path(self) -> str:
        """General Purpose: The sqlite3 file. The default is pyrbac.sqlite3 in the home folder."""
        override = self.get("PYRBAC_SQLITE_FILE")
        if override:
            return override
        folder = self.app_home()
        if not folder:
            raise SettingsError(
                "The home folder is not set.",
                "export HOME or PYRBAC_HOME",
                logger=self.logger,
            )
        return os.path.join(folder, self.SQLITE_NAME)

    def storage_label(self) -> str:
        """General Purpose: The sqlite3 path, or the MySQL database name."""
        if self.engine() == "sqlite3":
            return self.sqlite_path()
        return self.get("PYRBAC_DB_NAME")

    def choice_lines(self) -> list:
        """General Purpose: The two engine choices, including the default sqlite3 file."""
        path = self.sqlite_path()
        mark = ""
        if self.get("PYRBAC_SQLITE_FILE") == "":
            mark = " (default file)"
        return [
            "Database engine. Enter keeps the current engine.",
            "1 sqlite3  {0}{1}".format(path, mark),
            "2 mysql    {0}".format(self.get("PYRBAC_DB_NAME")),
            "Current: {0}".format(self.engine()),
        ]

    def engine_sentence(self) -> str:
        """General Purpose: One sentence naming the engine that the next connection uses."""
        if self.engine() == "sqlite3":
            return "Database engine: sqlite3 {0}.".format(self.sqlite_path())
        return "Database engine: mysql {0}.".format(self.get("PYRBAC_DB_NAME"))

    def apply_choice(self, token: str) -> str:
        """General Purpose: Keep, or store sqlite3 or mysql. The environment wins over the file."""
        text = "" if token is None else str(token).strip().lower()
        if text == "":
            return self.engine_sentence()
        if text in ("1", "sqlite3"):
            engine = "sqlite3"
        elif text in ("2", "mysql"):
            engine = "mysql"
        else:
            raise SettingsError(
                "The database engine must be sqlite3 or mysql.",
                "choose 1 for the sqlite3 file or 2 for mysql",
                logger=self.logger,
            )
        env = self.environ.get("PYRBAC_DB_ENGINE", "").strip().lower()
        if env:
            if env not in self.ENGINES:
                raise SettingsError(
                    "PYRBAC_DB_ENGINE must be sqlite3 or mysql.",
                    "unset PYRBAC_DB_ENGINE, or set sqlite3 or mysql",
                    logger=self.logger,
                )
            if env != engine:
                raise SettingsError(
                    "PYRBAC_DB_ENGINE is set in the environment.",
                    "unset PYRBAC_DB_ENGINE to store the choice in the database file",
                    logger=self.logger,
                )
            return self.engine_sentence() + " The environment wins."
        self.save_engine(engine)
        return self.engine_sentence()

    def save_engine(self, engine: str) -> None:
        """General Purpose: Store the engine in the mode-0600 file. Other lines stay."""
        engine = "" if engine is None else str(engine).strip().lower()
        if engine not in self.ENGINES:
            raise SettingsError(
                "The database engine must be sqlite3 or mysql.",
                "choose 1 for the sqlite3 file or 2 for mysql",
                logger=self.logger,
            )
        folder = self.app_home()
        if not folder:
            raise SettingsError(
                "The home folder is not set.",
                "export HOME or PYRBAC_HOME",
                logger=self.logger,
            )
        os.makedirs(folder, mode=0o700, exist_ok=True)
        os.chmod(folder, 0o700)
        path = self.database_file()
        lines = []
        if os.path.isfile(path):
            mode = os.stat(path).st_mode & 0o777
            if mode != 0o600:
                raise SettingsError(
                    "The database file is not mode 0600.",
                    "chmod 0600 the database file, or remove it and choose again",
                    logger=self.logger,
                )
            try:
                with open(path, "r", encoding="utf-8") as handle:
                    lines = handle.read().splitlines()
            except OSError as exc:
                raise SettingsError(
                    "The database file could not be read.",
                    "check the database file, then choose the engine again",
                    logger=self.logger,
                ) from exc
        found = False
        rewritten = []
        for line in lines:
            key = line.split("=", 1)[0].strip() if "=" in line else ""
            if key == "PYRBAC_DB_ENGINE":
                rewritten.append("PYRBAC_DB_ENGINE={0}".format(engine))
                found = True
            else:
                rewritten.append(line)
        if not found:
            rewritten.append("PYRBAC_DB_ENGINE={0}".format(engine))
        payload = ("\n".join(rewritten).rstrip("\n") + "\n").encode("utf-8")
        temporary = path + ".tmp"
        descriptor = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
        try:
            os.write(descriptor, payload)
        finally:
            os.close(descriptor)
        os.chmod(temporary, 0o600)
        os.replace(temporary, path)
        os.chmod(path, 0o600)
        self._file_cache = None

    def password(self) -> str:
        """General Purpose: The database password. Empty when neither source has it."""
        return self.get("PYRBAC_DB_PASSWORD")

    def require_password(self) -> str:
        value = self.password()
        if value:
            return value
        raise SettingsError(
            "The database password is not set.",
            "export PYRBAC_DB_PASSWORD and run py-rbac db-init",
            logger=self.logger,
        )

    def database(self) -> dict:
        """General Purpose: The MySQL connection values. The password is required."""
        name = self.get("PYRBAC_DB_NAME")
        if not name or not all(ch.isalnum() or ch == "_" for ch in name):
            raise SettingsError(
                "PYRBAC_DB_NAME must be letters, digits, and underscores.",
                "export PYRBAC_DB_NAME=pyrbac",
                logger=self.logger,
            )
        try:
            port = int(self.get("PYRBAC_DB_PORT"))
        except ValueError:
            port = 0
        if port < 1 or port > 65535:
            raise SettingsError(
                "PYRBAC_DB_PORT must be a port number.",
                "export PYRBAC_DB_PORT=3306",
                logger=self.logger,
            )
        return {
            "host": self.get("PYRBAC_DB_HOST"),
            "port": port,
            "user": self.get("PYRBAC_DB_USER"),
            "password": self.require_password(),
            "database": name,
        }

    def bind(self) -> tuple:
        """General Purpose: Web bind address and port. Non-loopback needs an explicit allow."""
        host = self.get("PYRBAC_BIND")
        try:
            port = int(self.get("PYRBAC_PORT"))
        except ValueError:
            port = -1
        if port < 0 or port > 65535:
            raise SettingsError(
                "PYRBAC_PORT must be a port number. 0 asks the system for a free port.",
                "export PYRBAC_PORT=8765",
                logger=self.logger,
            )
        if host not in self.LOOPBACK and self.environ.get("PYRBAC_ALLOW_REMOTE", "") != "yes":
            raise SettingsError(
                "Refusing to listen on {0}.".format(host),
                "keep PYRBAC_BIND=127.0.0.1, or export PYRBAC_ALLOW_REMOTE=yes",
                logger=self.logger,
            )
        return host, port
