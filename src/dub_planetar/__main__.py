#***********************************************
#* (c) Créations Daniel Dubé     Daniel Dubé   *
#* Dernières Modifications -->   2026-07-21    *
#***********************************************
from __future__ import annotations

import sys

from dub_planetar.platform_env import install_script_command


def main() -> None:
    try:
        from PySide6.QtWidgets import QApplication
    except ImportError:
        print(
            "ERREUR : PySide6 n'est pas installé.\n"
            f"Exécutez : {install_script_command()}",
            file=sys.stderr,
        )
        raise SystemExit(1) from None

    from dub_planetar.app import run_app
    from dub_planetar.preflight import show_preflight_dialog

    app = QApplication(sys.argv)
    if not show_preflight_dialog(app):
        raise SystemExit(1)
    run_app(existing_app=app)


if __name__ == "__main__":
    main()
