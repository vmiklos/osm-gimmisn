#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The ranges module contains functionality related to the Ranges class."""

from typing import List
from typing import Optional
from typing import cast


class Range:
    """A range object represents an odd or even range of integer numbers."""
    def __init__(self, start: int, end: int, interpolation: str = "") -> None:
        self.__start = start
        self.__end = end
        self.__is_odd: Optional[bool] = start % 2 == 1
        if interpolation == "all":
            self.__is_odd = None

    def get_start(self) -> int:
        """The smallest integer."""
        return self.__start

    def get_end(self) -> int:
        """The largest integer."""
        return self.__end

    def is_odd(self) -> Optional[bool]:
        """None for all house numbers on one side, bool otherwise."""
        return self.__is_odd

    def __contains__(self, item: int) -> bool:
        if (self.__is_odd is not None) and self.__is_odd != (item % 2 == 1):
            return False
        if self.__start <= item <= self.__end:
            return True
        return False

    def __repr__(self) -> str:
        return "Range(start=%s, end=%s, is_odd=%s)" % (self.__start, self.__end, self.__is_odd)

    def __eq__(self, other: object) -> bool:
        other_range = cast(Range, other)
        if self.__start != other_range.get_start():
            return False
        if self.__end != other_range.get_end():
            return False
        if self.__is_odd != other_range.is_odd():
            return False
        return True


class Ranges:
    """A Ranges object contains an item if any of its Range objects contains it."""
    def __init__(self, items: List[Range]) -> None:
        self.__items = items

    def get_items(self) -> List[Range]:
        """The list of contained Range objects."""
        return self.__items

    def __contains__(self, item: int) -> bool:
        for i in self.__items:
            if item in i:
                return True
        return False

    def __repr__(self) -> str:
        return "Ranges(items=%s)" % self.__items

    def __eq__(self, other: object) -> bool:
        other_ranges = cast(Ranges, other)
        return self.__items == other_ranges.get_items()


# vim:set shiftwidth=4 softtabstop=4 expandtab:
