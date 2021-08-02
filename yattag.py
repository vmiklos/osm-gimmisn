#!/usr/bin/env python3
#
# Copyright (c) 2021 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""
The yattag module generates HTML with Python.

This is a stripped down version of the Python package.
"""

from typing import Any
from typing import List
from typing import Tuple


class Doc:
    """Generates xml/html documents."""
    def __init__(self) -> None:
        self.value = ""

    def get_value(self) -> str:
        """Gets the escaped value."""
        return self.value

    def append_value(self, value: str) -> None:
        """Appends escaped content to the value."""
        self.value += value

    def tag(self, name: str, attrs: List[Tuple[str, str]]) -> 'Tag':
        """Starts a new tag."""
        return Tag(self, name, attrs)

    def stag(self, name: str, attrs: List[Tuple[str, str]]) -> None:
        """Starts a new tag and closes it as well."""
        self.append_value("<{}".format(name))
        for attr in attrs:
            key = attr[0]
            value = attr[1].replace("&", "&amp;").replace("<", "&lt;").replace('"', "&quot;")
            self.append_value(" {}=\"{}\"".format(key, value))
        self.append_value(" />")

    def text(self, text: str) -> None:
        """Appends unescaped content to the document."""
        self.append_value(text.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;"))


class Tag:
    """Starts a tag, which is closed automatically."""
    def __init__(self, doc: Doc, name: str, attrs: List[Tuple[str, str]]) -> None:
        doc.append_value("<{}".format(name))
        for attr in attrs:
            key = attr[0]
            value = attr[1].replace("&", "&amp;").replace("<", "&lt;").replace('"', "&quot;")
            doc.append_value(" {}=\"{}\"".format(key, value))
        doc.append_value(">")
        self.doc = doc
        self.name = name

    def __enter__(self) -> None:
        pass

    def __exit__(self, tpe: Any, value: Any, traceback: Any) -> None:
        self.doc.append_value("</{}>".format(self.name))


# vim:set shiftwidth=4 softtabstop=4 expandtab:
