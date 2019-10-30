#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The i18n module allows UI translation via gettext."""


from typing import cast
import gettext
import threading


def set_language(language: str) -> None:
    """Sets the language of the current thread."""
    tls = threading.current_thread.__dict__
    import helpers
    localedir = helpers.get_abspath("locale")
    tls["translations"] = gettext.translation("osm-gimmisn", localedir=localedir, languages=[language], fallback=True)


def translate(fro: str) -> str:
    """Translates input according to the current UI language."""
    tls = threading.current_thread.__dict__
    if "translations" not in tls.keys():
        return fro

    return cast(str, tls["translations"].gettext(fro))

# vim:set shiftwidth=4 softtabstop=4 expandtab:
