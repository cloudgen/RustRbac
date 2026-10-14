# =============================================================================
# Text menu session for PyRbac.
# requirement-python-tui — menu region and the bottom input box.
# requirement-python-about — the about result page is framework_about.
# requirement-python-oop — this module defines class Tui and MenuScreenError.
# The frame is class MenuPainter. cli.py constructs Tui and calls it.
# =============================================================================
from __future__ import annotations

import curses
from types import SimpleNamespace

from .menu_painter import MenuPainter
from .menu_session import MenuSession
from .system_log import SystemLog



class MenuScreenError(Exception):
    """The terminal cannot hold the menu and the framed input box."""

    def __init__(self, message="", logger=None):
        super().__init__(message)
        self.logger = logger
        if logger is not None:
            logger.log_message("instantiated", component="MenuScreenError")


class Tui:
    """Text menu session. The front board and the result pages stay on this screen.

    MenuModel, MenuSession, and MenuPainter live in their own modules.
    The rows are access, system-log, language, setup, audit, serve,
    self-management, and Exit. There is no session row. Empty arguments ask
    for an account and a password before that board. Domain rows call Desk.
    """

    def __init__(self, app=None, app_name="PyRbac", version=None, logger=None):
        self.logger = logger
        if logger is not None:
            logger.log_message("instantiated", component="Tui")
        self.app = app
        self.app_name = app_name
        if version is None:
            from . import __version__
            version = __version__
        self.version = version
        self.painter = MenuPainter(logger=logger)
        self.system_log = SystemLog(logger=logger)

    def _name(self):
        if self.app is not None:
            return self.app.app_name
        return self.app_name

    def _ver(self):
        if self.app is not None:
            return self.app.version
        return self.version

    def front_lines(self):
        """General Purpose: Path line and the front board rows."""
        return [self.painter.path_line()] + self.painter.format_rows(
            self.painter.display_rows("front")
        )

    def self_lines(self):
        """General Purpose: Path line and the self-management board under 8."""
        return [self.painter.path_line()] + self.painter.format_rows(
            self.painter.display_rows("self")
        )

    def log_lines(self):
        """General Purpose: Path line and the system-log board under 3."""
        return [self.painter.path_line()] + self.painter.format_rows(
            self.painter.display_rows("log")
        )

    def language_lines(self):
        """General Purpose: Path line and the language board under 4."""
        return [self.painter.path_line()] + self.painter.format_rows(
            self.painter.display_rows("lang")
        )

    def help_on_screen(self):
        """
        General Purpose: Help text when help is typed on the open menu.
        The typed py-rbac help verb does not call this and does not draw the screen.
        """
        name = self._name()
        verbs = ", ".join(self.app.PRODUCT_VERBS) if self.app is not None else "help, version, about"
        return (
            "help: usage for {0}.\n"
            "Verbs: {1}.\n"
            "version shows the installed version and does not call pip.\n"
            "about shows the about page.\n"
            "Sign in before this menu. login, logout, and whoami are typed verbs.\n"
            "access, setup, and audit repeat the web pages. serve starts the site.\n"
            "This program does not join media or list files in this folder.\n"
            "version-check runs: python -m pip index versions {0}\n"
            "self-update runs: python -m pip install --upgrade {0}\n"
            "self-install runs: python -m pip install {0}\n"
            "self-uninstall runs: python -m pip uninstall -y {0}\n"
            "On the command line, self-uninstall also needs --force.\n"
            "Empty arguments on a terminal open this menu and do not run pip.\n"
            "--password is visible in the process list. This menu masks it."
        ).format(name, verbs)

    def about_text(self):
        """General Purpose: The about page body from AboutPage.framework_about."""
        if self.app is None:
            return ""
        return self.app.about.framework_about()

    def _visible_lines(self, lines, room, pin_last):
        """Lines that fit above the input box. A question pins its last line."""
        if room < 1:
            return []
        if len(lines) <= room:
            return list(lines)
        if not pin_last:
            return list(lines[-room:])
        if room == 1:
            return [lines[-1]]
        return list(lines[: room - 2]) + ["…", lines[-1]]

    def _paint_lines(self, screen, model, title, lines, pin_last):
        """Hand one question to MenuPainter. The session does not draw a second frame."""
        self.painter.paint_prompt(
            screen,
            model,
            title,
            lines,
            pin_last,
            self._name(),
            self._ver(),
            self._visible_lines,
        )

    def _arm_blocking(self, screen):
        """Clear the one-second clock wait. A no-key from that wait is not Esc."""
        arm = getattr(screen, "timeout", None)
        if arm is not None:
            arm(-1)

    def read_key(self, screen, model, lines, title="menu"):
        """
        General Purpose: Read one line from the bottom input box.

        Returns the text, or None when the person presses Esc. The board's
        one-second clock wait is cleared before each key.
        """
        model.buffer = ""
        model.cursor = 0
        model.focus = "input"
        while True:
            self._paint_lines(screen, model, title, lines, pin_last=True)
            self._arm_blocking(screen)
            key = screen.getch()
            if key in (-1, 27):
                model.error = ""
                model.buffer = ""
                model.cursor = 0
                model.focus = "list"
                return None
            if key in (10, 13, curses.KEY_ENTER):
                text = model.buffer
                model.buffer = ""
                model.cursor = 0
                model.error = ""
                return text
            if key in (curses.KEY_BACKSPACE, 127, 8):
                model._backspace()
                model.error = ""
                continue
            if key in (curses.KEY_LEFT, curses.KEY_RIGHT) or key in model.edge_keys():
                model._slide(key)
                continue
            if key in (curses.KEY_UP, curses.KEY_DOWN):
                continue
            if 32 <= key < 127:
                model._insert(chr(key))
                model.error = ""
                continue

    def notice(self, screen, model, lines, title="result", closing=None):
        """General Purpose: Show lines until the next key. The clock wait is cleared."""
        model.buffer = ""
        model.cursor = 0
        model.focus = "list"
        model.error = ""
        if closing is None:
            closing = "Press a key to return to the main menu."
        body = list(lines) + ["", closing]
        self._paint_lines(screen, model, title, body, pin_last=True)
        self._arm_blocking(screen)
        try:
            screen.getch()
        except Exception:
            return

    def _read_yes_no(self, screen, model, lines, title="system-log"):
        """y/n from the box. Esc returns None. Empty is no."""
        while True:
            raw = self.read_key(screen, model, lines, title=title)
            if raw is None:
                return None
            raw = raw.strip().lower()
            if raw == "" or raw in ("n", "no"):
                return False
            if raw in ("y", "yes"):
                return True
            model.error = "Please enter y or n"

    def _read_index(self, screen, model, lines, count, title="system-log"):
        """1-based index from the box. Ask again on a bad value. Esc returns None."""
        while True:
            raw = self.read_key(screen, model, lines, title=title)
            if raw is None:
                return None
            raw = raw.strip()
            try:
                idx = int(raw)
            except ValueError:
                model.error = "Please type a number"
                continue
            if 1 <= idx <= count:
                return idx - 1
            model.error = "Enter 1–{0}".format(count)

    def about_on_screen(self, screen=None, model=None):
        """
        General Purpose: Show the about page.
        With no screen, open the text screen when this is the direct verb.
        """
        text = self.about_text()
        if screen is not None and model is not None:
            self.notice(screen, model, text.splitlines() or [""], title="about")
            return 0
        if self.app is not None and self.app.stdout_is_tty():
            return self.open_direct("about")
        return text

    def _screen_error(self, too_small):
        if self.app is None:
            return "missing"
        if too_small:
            self.app.report_error(
                "The text screen is too small for the menu and the input box.",
                "{0} help".format(self.app.CONSOLE_NAME),
            )
        else:
            self.app.report_error(
                "The text menu could not open on this terminal.",
                "{0} help".format(self.app.CONSOLE_NAME),
            )
        return "missing"

    def _require_terminal(self):
        if self.app is None or not self.app.stdout_is_tty():
            if self.app is not None:
                self.app.report_error(
                    "No terminal for the text menu. Use a terminal.",
                    "{0} help".format(self.app.CONSOLE_NAME),
                )
            return False
        return True

    def login_lines(self, state="", error="", database_name=""):
        """General Purpose: The sign-in text shown before the front board."""
        lines = [
            "Sign in before the menu.",
            "Type the account name, then the password.",
            "The password is masked. Esc leaves.",
        ]
        if state == "seeded" and database_name:
            lines.append("Database {0}: seeded.".format(database_name))
            lines.append("Seeded accounts are in the product README. Change them after this sign-in.")
        if error:
            lines.append("")
            lines.append("ERROR: {0}".format(error))
        return lines

    def login_gate(self, ask, state="present", database_name=""):
        """General Purpose: Ask until the password matches. Esc returns False.

        ask(lines) returns a dict with user_name and password, or None.
        """
        error = ""
        while True:
            picked = ask(self.login_lines(state, error, database_name))
            if picked is None:
                return False
            if self.app is None or getattr(self.app, "desk", None) is None:
                error = "This menu has no command desk."
                continue
            try:
                self.app.desk.login(
                    picked.get("user_name", ""),
                    picked.get("password", ""),
                )
                return True
            except Exception as exc:
                error = str(exc)

    def _menu_engine(self, screen, model):
        """Ask for sqlite3 or mysql. Esc returns None. Enter keeps the current engine."""
        while True:
            lines = list(self.app.desk.settings.choice_lines())
            lines.append("1 sqlite3 file, 2 mysql, or Enter to keep. Esc leaves.")
            text = self.read_key(screen, model, lines, title="database")
            if text is None:
                return None
            try:
                return self.app.desk.settings.apply_choice(text.strip())
            except Exception as exc:
                nxt = getattr(exc, "nxt", "py-rbac db-engine")
                self.notice(
                    screen,
                    model,
                    ["ERROR: {0}".format(exc), "Next: {0}".format(nxt)],
                    title="database",
                    closing="Press a key to choose again.",
                )

    def open_text_menu(self):
        """
        General Purpose: Ask for the database engine, initialize a missing
        database, ask for a sign-in, then draw the front board. Does not run
        pip. Returns "missing" when the screen cannot open or the database
        cannot be prepared.
        """
        if not self._require_terminal():
            return "missing"
        if self.app is None or getattr(self.app, "desk", None) is None:
            return self._screen_error(False)
        try:
            import curses
        except ImportError:
            return self._screen_error(False)

        screen_box = {}
        error_box = {}

        def on_kind(kind):
            if kind == "help":
                return self.help_on_screen()
            if kind in ("version-check", "self-update", "self-install", "self-uninstall"):
                _code, text = self.app.self_manage.run_pip(kind)
                return text
            if kind == "view-log":
                screen = screen_box.get("screen")
                if screen is None:
                    return None
                return self._view_log(screen, session.model)
            if kind == "clear-log":
                screen = screen_box.get("screen")
                if screen is None:
                    return None
                return self._clear_log(screen, session.model)
            if kind == "log-folder":
                return self._log_folder()
            if kind in self.DOMAIN_KINDS:
                screen = screen_box.get("screen")
                if screen is None:
                    return None
                return self._domain(kind, screen, session.model)
            return None

        session = MenuSession(
            self._name(),
            self._ver(),
            on_version=lambda: self.app.self_manage.local_version(),
            on_about=self.about_text,
            boards=None,
            on_kind=on_kind,
            logger=self.logger,
            painter=self.painter,
        )

        def _wrapped(screen):
            screen_box["screen"] = screen
            try:
                if self._menu_engine(screen, session.model) is None:
                    return
                state = self.app.desk.ensure_database()
                database_name = self.app.desk.settings.storage_label()
            except Exception as exc:
                error_box["exc"] = exc
                return
            signed_in = self.login_gate(
                lambda lines: self._ask_fields(
                    screen, session.model, "login", self.ASKS["login"],
                ),
                state=state,
                database_name=database_name,
            )
            if not signed_in:
                return
            session.run(screen)

        try:
            curses.wrapper(_wrapped)
        except MenuScreenError:
            return self._screen_error(True)
        except Exception:
            return self._screen_error(False)
        if "exc" in error_box:
            exc = error_box["exc"]
            nxt = getattr(exc, "nxt", "py-rbac db-init")
            self.app.report_error(str(exc), nxt)
            return "missing"
        return None

    def open_direct(self, kind):
        """
        General Purpose: Open the text screen on about.
        Does not start on the front board. Returns a process status.
        """
        if not self._require_terminal():
            return 1
        try:
            import curses
        except ImportError:
            self._screen_error(False)
            return 1

        code_box = {"code": 0}
        session = MenuSession(
            self._name(),
            self._ver(),
            on_version=lambda: self.app.self_manage.local_version(),
            on_about=self.about_text,
            boards=None,
            on_kind=lambda _kind: None,
            logger=self.logger,
            painter=self.painter,
        )

        def _wrapped(screen):
            try:
                if kind != "about":
                    code_box["code"] = 1
                    return
                code_box["code"] = self.about_on_screen(screen, session.model)
            except Exception as exc:
                self.notice(
                    screen, session.model, ["ERROR: {0}".format(exc)],
                    closing="Press a key to close.",
                )
                code_box["code"] = 1
            session.leave_to_front()
            # Esc and a finished page return to the front board.
            if code_box["code"] == 0:
                session.run(screen)

        try:
            curses.wrapper(_wrapped)
        except MenuScreenError:
            self._screen_error(True)
            return 1
        except Exception:
            self._screen_error(False)
            return 1
        return code_box["code"]

    def _view_log(self, screen, model):
        """List log files and return the chosen file for the result page."""
        folder = self.system_log.log_dir()
        if not folder:
            return "No log folder."
        files = self.system_log.log_files()
        if not files:
            return "No log file in {0}.".format(folder)
        lines = ["Log files:"]
        for index, path in enumerate(files, start=1):
            lines.append("{0}. {1}".format(index, path.name))
        lines.append("")
        lines.append("Choose a log file:")
        picked = self._read_index(screen, model, lines, len(files), title="system-log")
        if picked is None:
            return None
        path = files[picked]
        body = self.system_log.read_log(path)
        if body is None:
            return "ERROR: {0} is not a log file in the log folder.".format(path.name)
        if body == "":
            body = "(empty)"
        return "{0}\n\n{1}".format(path.name, body)

    def _clear_log(self, screen, model):
        """List log files, confirm, and empty the chosen file."""
        folder = self.system_log.log_dir()
        if not folder:
            return "No log folder."
        files = self.system_log.log_files()
        if not files:
            return "No log file in {0}.".format(folder)
        lines = ["Log files:"]
        for index, path in enumerate(files, start=1):
            lines.append("{0}. {1}".format(index, path.name))
        lines.append("")
        lines.append("Choose a log file to clear:")
        picked = self._read_index(screen, model, lines, len(files), title="system-log")
        if picked is None:
            return None
        path = files[picked]
        answer = self._read_yes_no(
            screen,
            model,
            ["Clear {0}? (y/n)".format(path.name)],
            title="system-log",
        )
        if answer is not True:
            return None
        if not self.system_log.clear_log(path):
            return "ERROR: {0} is not a log file in the log folder.".format(path.name)
        return "Cleared {0}.".format(path.name)

    def _log_folder(self):
        """The log-folder result page. The path is logDir()."""
        return self.system_log.folder_text()

    DOMAIN_KINDS = (
        "login",
        "whoami",
        "logout",
        "serve",
        "user-list",
        "user-add",
        "user-edit",
        "user-remove",
        "user-reset-password",
        "role-list",
        "role-grant",
        "menu-list",
        "menu-add",
        "dept-list",
        "dept-add",
        "post-list",
        "post-add",
        "config-list",
        "config-set",
        "config-add",
        "dict-list",
        "dict-add",
        "notice-list",
        "notice-add",
        "oper-log",
        "login-log",
        "online-list",
        "online-logout",
        "db-engine",
    )
    ASKS = {
        "login": (("user_name", "user_name", False), ("password", "password", True)),
        "user-add": (
            ("user_name", "user_name", False),
            ("nick_name", "nick_name", False),
            ("dept_id", "dept_id", False),
            ("role_id", "role_id", False),
            ("password", "password, or empty for the initial password", True),
        ),
        "user-edit": (
            ("user_id", "user_id", False),
            ("nick_name", "nick_name, or empty to keep it", False),
            ("dept_id", "dept_id, or empty to keep it", False),
            ("status", "status 0 or 1, or empty to keep it", False),
        ),
        "user-remove": (("user_id", "user_id", False),),
        "user-reset-password": (("user_id", "user_id", False), ("password", "password", True)),
        "role-grant": (
            ("role_id", "role_id", False),
            ("menu_ids", "menu ids separated by commas", False),
            ("data_scope", "data_scope 1-4, or empty to keep it", False),
            ("dept_ids", "dept ids for scope 2, or empty", False),
        ),
        "menu-add": (
            ("menu_name", "menu_name", False),
            ("parent_id", "parent_id", False),
            ("menu_type", "menu_type M, C, or F", False),
            ("perms", "permission string, or empty", False),
            ("path", "path, or empty", False),
        ),
        "dept-add": (("dept_name", "dept_name", False), ("parent_id", "parent_id", False)),
        "post-add": (("post_code", "post_code", False), ("post_name", "post_name", False)),
        "config-set": (("config_key", "config_key", False), ("config_value", "config_value", False)),
        "config-add": (
            ("config_key", "config_key", False),
            ("config_name", "config_name", False),
            ("config_value", "config_value", False),
        ),
        "dict-add": (
            ("dict_type", "dict_type", False),
            ("dict_label", "dict_label", False),
            ("dict_value", "dict_value", False),
        ),
        "notice-add": (
            ("notice_title", "notice title", False),
            ("notice_type", "notice_type 1 or 2", False),
            ("content", "content, or empty", False),
        ),
        "online-logout": (("token", "session token", False),),
    }

    def _ask_fields(self, screen, model, title, fields):
        """One field at a time. Esc returns None. A password field is masked."""
        values = {}
        for name, label, secret in fields:
            model.mask = bool(secret)
            try:
                text = self.read_key(screen, model, [label], title=title)
            finally:
                model.mask = False
            if text is None:
                return None
            values[name] = text.strip()
        return values

    def _domain(self, kind, screen, model):
        """General Purpose: The same Desk verb the command line runs."""
        if self.app is None or getattr(self.app, "desk", None) is None:
            return "ERROR: This menu has no command desk.\nNext: py-rbac help"
        if kind == "db-engine":
            return self._menu_engine(screen, model)
        options = SimpleNamespace()
        fields = self.ASKS.get(kind)
        if fields:
            picked = self._ask_fields(screen, model, kind, fields)
            if picked is None:
                return None
            for name, value in picked.items():
                setattr(options, name, value)
        try:
            if kind == "serve":
                return self.app.desk.serve_background()
            return self.app.desk.execute(kind, options)
        except Exception as exc:
            nxt = getattr(exc, "nxt", "py-rbac help")
            return "ERROR: {0}\nNext: {1}".format(exc, nxt)
