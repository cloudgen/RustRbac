#!/usr/bin/env python
# =============================================================================
# PyRbac CLI — text menu and typed verbs.
# Law keys: requirement-python-cli-interface, requirement-python-cli-logging,
#           requirement-python-tui, requirement-python-oop,
#           requirement-python-about, requirement-domain-pyrbac,
#           requirement-python-error-handling, requirement-python-coding-style,
#           requirement-runtime-prerequisites
# Video join, FFmpeg, and file listing are not part of this program.
# CIAO-Lite: Caution • Intentional • Anti-fragile • Over-protect
# def main writes ChronicleLogger(...). This module does not construct it
# at import time and does not re-export it.
# =============================================================================
from __future__ import print_function, unicode_literals

import argparse
import sys

from .about_page import AboutPage
from .check_system import CheckSystem
from .desk import AccessError, Desk, SettingsError, StoreError
from .self_management import SelfManage
from .tui import Tui


class Cli:
    """Parser, dispatch, verbs, and the tty gate. One class in this file.

    Identity, verb lists, and suffix sets are attributes of this class.
    Collaborators arrive through the constructor. def main stays beside this class.
    """

    from . import __version__ as VERSION

    APP_NAME = "PyRbac"
    CONSOLE_NAME = "py-rbac"
    PRODUCT_VERBS = (
        "help",
        "version",
        "about",
        "self-install",
        "version-check",
        "self-update",
        "self-uninstall",
        "login",
        "logout",
        "whoami",
        "serve",
        "db-init",
        "db-engine",
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
    )
    LIFECYCLE_VERBS = (
        "version",
        "self-install",
        "version-check",
        "self-update",
        "self-uninstall",
    )
    DOMAIN_VERBS = (
        "login",
        "logout",
        "whoami",
        "db-init",
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
    )
    AUTHOR_NAME = "Wong Chun Fai"
    AUTHOR_EMAIL = "cloudgen.wong@gmail.com"
    HOMEPAGE = "https://github.com/cloudgen/PyRbac"
    LAST_UPDATE = "2026-10-11"
    DOWNLOAD_URL = ""
    BASIC_USAGE = "py-rbac login --user-name admin"

    def __init__(self, logger=None):
        self.logger = logger
        if logger is not None:
            logger.log_message("instantiated", component="Cli")
        self.app_name = Cli.APP_NAME
        self.version = Cli.VERSION
        self.about = AboutPage(
            CheckSystem(
                logger=logger,
                app_name=Cli.APP_NAME,
                version=Cli.VERSION,
                console_name=Cli.CONSOLE_NAME,
            ),
            Cli.APP_NAME,
            Cli.VERSION,
            Cli.AUTHOR_NAME,
            Cli.LAST_UPDATE,
            Cli.HOMEPAGE,
            Cli.DOWNLOAD_URL,
            Cli.BASIC_USAGE,
            Cli.CONSOLE_NAME,
            logger=logger,
        )
        self.self_manage = SelfManage(Cli.APP_NAME, Cli.VERSION, logger=logger)
        self.desk = Desk(logger=logger)
        self.tui = Tui(self, logger=logger)

    @staticmethod
    def stdout_is_tty():
        """General Purpose: Whether the text screen can be drawn on this stdout."""
        try:
            return sys.stdout.isatty()
        except Exception:
            return False

    @staticmethod
    def opens_text_screen(argv):
        """
        General Purpose: Whether this argv draws the text screen.

        The real parser has not run yet. This walk only decides is_quiet.
        Help, version, the domain verbs, and the pip verbs do not draw the screen.
        Empty argv and about do, when stdout is a terminal.
        """
        if argv is None:
            argv = sys.argv[1:]
        argv = list(argv)
        if "--help" in argv or "-h" in argv or "--version" in argv:
            return False
        if not Cli.stdout_is_tty():
            return False
        verb = None
        saw_force = False
        for token in argv:
            if token == "--force":
                saw_force = True
                continue
            if token.startswith("-"):
                return False
            if verb is None:
                verb = token
                continue
            return False
        if saw_force and verb != "self-uninstall":
            return False
        if verb == "about":
            return True
        return verb is None

    def report_error(self, message, nxt):
        """User-visible failure plus the same fact on the logger when one exists."""
        if self.logger is not None:
            self.logger.log_message(
                "{0} Next: {1}".format(message, nxt),
                level="ERROR",
                component="main",
            )
        print("ERROR: {0}".format(message), file=sys.stderr)
        print("   Next: {0}".format(nxt), file=sys.stderr)
        return 1

    def build_parser(self):
        """
        General Purpose: One optional product verb. No file-operand flags.
        help is also --help. --force belongs only to self-uninstall.
        """
        verbs = ", ".join(Cli.PRODUCT_VERBS)
        parser = argparse.ArgumentParser(
            prog=Cli.CONSOLE_NAME,
            formatter_class=argparse.RawDescriptionHelpFormatter,
            description=(
                "{0} — online RBAC and centralized configuration.\n"
                "With no arguments on a terminal, asks for the database engine,\n"
                "then an account and a password, then opens the text menu.\n"
                "The default engine is the sqlite3 file. It creates missing tables first.\n"
                "db-engine chooses sqlite3 or mysql and does not open the database.\n"
                "With no arguments and no terminal, prints this help and stops.\n"
                "Product verbs: {1}.\n"
                "The text menu has no session row. Row 7 serve starts the site.\n"
                "This program does not join media or list files in this folder.\n"
                "version-check runs: python -m pip index versions {0}\n"
                "self-update runs: python -m pip install --upgrade {0}\n"
                "self-install runs: python -m pip install {0}\n"
                "self-uninstall runs: python -m pip uninstall -y {0}\n"
                "and needs --force. Empty arguments do not install or update.\n"
                "--password is visible in the process list. The menu masks it.\n"
                "Change the seeded passwords after the first sign-in."
                .format(Cli.APP_NAME, verbs)
            ),
        )
        parser.add_argument(
            "verb",
            nargs="?",
            default=None,
            metavar="verb",
            help="Product verb. py-rbac help lists them.",
        )
        parser.add_argument(
            "--version",
            action="version",
            version="{0} {1}".format(Cli.APP_NAME, Cli.VERSION),
        )
        parser.add_argument(
            "--force",
            action="store_true",
            help="Confirm self-uninstall. Required on the command line.",
        )
        parser.add_argument("--user-name", default="", help="Login name.")
        parser.add_argument(
            "--password",
            default="",
            help="Password. Visible in the process list. The menu masks this field.",
        )
        parser.add_argument("--nick-name", default="")
        parser.add_argument("--dept-id", default="")
        parser.add_argument("--role-id", default="")
        parser.add_argument("--user-id", default="")
        parser.add_argument("--status", default="", help="0 normal, 1 disabled.")
        parser.add_argument("--menu-ids", default="", help="Comma-separated menu ids.")
        parser.add_argument("--data-scope", default="", help="1 all, 2 custom, 3 this department, 4 children.")
        parser.add_argument("--dept-ids", default="", help="Comma-separated department ids for data scope 2.")
        parser.add_argument("--menu-name", default="")
        parser.add_argument("--parent-id", default="")
        parser.add_argument("--menu-type", default="", help="M directory, C page, F button.")
        parser.add_argument("--perms", default="")
        parser.add_argument("--path", default="")
        parser.add_argument("--post-code", default="")
        parser.add_argument("--post-name", default="")
        parser.add_argument("--dept-name", default="")
        parser.add_argument("--config-key", default="")
        parser.add_argument("--config-name", default="")
        parser.add_argument("--config-value", default="")
        parser.add_argument("--dict-type", default="")
        parser.add_argument("--dict-label", default="")
        parser.add_argument("--dict-value", default="")
        parser.add_argument("--notice-title", default="")
        parser.add_argument("--notice-type", default="", help="1 notice, 2 announcement.")
        parser.add_argument("--content", default="")
        parser.add_argument("--token", default="", help="Online session token from online-list.")
        parser.add_argument(
            "--engine",
            default="",
            help="For db-engine: sqlite3 (the default file) or mysql.",
        )
        return parser

    def _page(self, text):
        print(text)
        return 0

    def _unknown_verb(self, token):
        names = ", ".join(Cli.PRODUCT_VERBS)
        return self.report_error(
            "Unknown verb '{0}'.".format(token),
            "{0} help — verbs: {1}".format(Cli.CONSOLE_NAME, names),
        )

    def _dispatch(self, args):
        """One product verb, the front board, or a fail-closed stop."""
        verb = args.verb
        if args.force and verb != "self-uninstall":
            return self.report_error(
                "--force is only for self-uninstall.",
                "{0} self-uninstall --force".format(Cli.CONSOLE_NAME),
            )
        if verb == "about":
            result = self.tui.about_on_screen()
            if isinstance(result, int):
                return result
            return self._page(result)
        if verb == "version":
            return self._page(self.self_manage.local_version())
        if verb == "self-uninstall":
            if not args.force:
                return self.report_error(
                    "self-uninstall removes this package with pip.",
                    "{0} self-uninstall --force".format(Cli.CONSOLE_NAME),
                )
            return self.self_manage.emit(verb)
        if verb in ("version-check", "self-update", "self-install"):
            return self.self_manage.emit(verb)
        if verb == "serve":
            return self._verb_serve()
        if verb == "db-engine":
            return self._verb_engine(args)
        if verb in Cli.DOMAIN_VERBS:
            return self._verb_domain(verb, args)
        if verb is not None:
            return self._unknown_verb(verb)
        if not self.stdout_is_tty():
            self.build_parser().print_help()
            return 0
        action = self.tui.open_text_menu()
        if action == "missing":
            return 1
        return 0

    def run(self, argv=None):
        """One job. def main already wrote ChronicleLogger(...) and passed it in."""
        if argv is None:
            argv = sys.argv[1:]
        argv = list(argv)
        parser = self.build_parser()
        try:
            args = parser.parse_args(argv)
        except SystemExit as exc:
            code = exc.code
            if code is None or code == 0:
                return 0
            return int(code)
        if args.verb == "help":
            parser.print_help()
            return 0
        return self._dispatch(args)

    def _verb_domain(self, verb, args):
        """One RBAC or setup verb. The menu calls the same Desk."""
        try:
            text = self.desk.execute(verb, args)
        except (AccessError, SettingsError, StoreError) as exc:
            return self.report_error(str(exc), exc.nxt)
        if text:
            print(text)
        return 0

    def _verb_engine(self, args):
        """Choose sqlite3 or mysql. This verb does not open the database."""
        settings = self.desk.settings
        if args.engine:
            try:
                print(settings.apply_choice(args.engine))
            except SettingsError as exc:
                return self.report_error(str(exc), exc.nxt)
            return 0
        try:
            lines = settings.choice_lines()
        except SettingsError as exc:
            return self.report_error(str(exc), exc.nxt)
        for line in lines:
            print(line)
        if not self.stdout_is_tty():
            return 0
        try:
            picked = input("Choice [keep]: ")
        except EOFError:
            return 0
        try:
            print(settings.apply_choice(picked))
        except SettingsError as exc:
            return self.report_error(str(exc), exc.nxt)
        return 0

    def _verb_serve(self):
        """Listen until interrupted. The menu uses a background listener instead."""
        try:
            self.desk.ensure_database()
            web = self.desk.web()
            web.make_server()
        except (SettingsError, StoreError, OSError) as exc:
            nxt = getattr(exc, "nxt", "keep PYRBAC_BIND=127.0.0.1")
            return self.report_error(str(exc), nxt)
        print(web.url())
        try:
            web.httpd.serve_forever()
        except KeyboardInterrupt:
            pass
        finally:
            web.stop()
        return 0


def main(argv=None, basedir="", logdir=""):
    """This function instantiates ChronicleLogger. The statement below is the construct."""
    if argv is None:
        argv = sys.argv[1:]
    argv = list(argv)
    try:
        from ChronicleLogger import ChronicleLogger
    except ImportError:
        sys.stderr.write(
            'ChronicleLogger is required. Next step: python -m pip install "ChronicleLogger>=1.3.1"\n'
        )
        return 1

    from PyRbac import __version__

    screen = Cli.opens_text_screen(argv)
    logger = ChronicleLogger(
        logname=Cli.APP_NAME,
        is_quiet=screen,
        basedir=basedir,
        logdir=logdir,
    )
    appname = logger.logName()
    resolved_base = logger.baseDir()
    logger.logDir()

    if logger.isDebug():
        logger.log_message(
            "{0} v{1} ({2})".format(appname, __version__, __file__),
            component="main",
        )
        logger.log_message(
            "Using {0}".format(ChronicleLogger.class_version()),
            component="main",
        )
        logger.log_message(
            "Base {0}".format(resolved_base),
            level="DEBUG",
            component="main",
        )
        logger.log_message("debug mode", component="main")

    app = Cli(logger)
    return app.run(argv)


if __name__ == "__main__":
    sys.exit(main())
