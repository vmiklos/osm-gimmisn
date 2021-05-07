#!/usr/bin/env python3
#
# Copyright (c) 2021 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The area_files module contains file handling functionality, to be used by the areas module."""

import os
from typing import TextIO
from typing import cast

import i18n
import util


class RelationFilePaths:
    """A relation's file interface provides access to files associated with a relation."""
    def __init__(self, datadir: str, workdir: str, name: str):
        self.__datadir = datadir
        self.__workdir = workdir
        self.__name = name

    def get_ref_streets_path(self) -> str:
        """Build the file name of the reference street list of a relation."""
        return os.path.join(self.__workdir, "streets-reference-%s.lst" % self.__name)

    def get_osm_streets_path(self) -> str:
        """Build the file name of the OSM street list of a relation."""
        return os.path.join(self.__workdir, "streets-%s.csv" % self.__name)

    def get_osm_housenumbers_path(self) -> str:
        """Build the file name of the OSM house number list of a relation."""
        return os.path.join(self.__workdir, "street-housenumbers-%s.csv" % self.__name)

    def get_ref_housenumbers_path(self) -> str:
        """Build the file name of the reference house number list of a relation."""
        return os.path.join(self.__workdir, "street-housenumbers-reference-%s.lst" % self.__name)

    def get_housenumbers_percent_path(self) -> str:
        """Builds the file name of the house number percent file of a relation."""
        return os.path.join(self.__workdir, "%s.percent" % self.__name)

    def get_housenumbers_htmlcache_path(self) -> str:
        """Builds the file name of the house number HTML cache file of a relation."""
        return os.path.join(self.__workdir, "%s.htmlcache.%s" % (self.__name, i18n.get_language()))

    def get_housenumbers_txtcache_path(self) -> str:
        """Builds the file name of the house number plain text cache file of a relation."""
        return os.path.join(self.__workdir, "%s.txtcache" % self.__name)

    def get_streets_percent_path(self) -> str:
        """Builds the file name of the street percent file of a relation."""
        return os.path.join(self.__workdir, "%s-streets.percent" % self.__name)

    def get_streets_additional_count_path(self) -> str:
        """Builds the file name of the street additional count file of a relation."""
        return os.path.join(self.__workdir, "%s-additional-streets.count" % self.__name)

    def get_housenumbers_additional_count_path(self) -> str:
        """Builds the file name of the housenumber additional count file of a relation."""
        return os.path.join(self.__workdir, "%s-additional-housenumbers.count" % self.__name)

    def get_additional_housenumbers_htmlcache_path(self) -> str:
        """Builds the file name of the additional house number HTML cache file of a relation."""
        return os.path.join(self.__workdir, "%s.additional-htmlcache.%s" % (self.__name, i18n.get_language()))


class RelationFiles(RelationFilePaths):
    """Extends RelationFilePaths with streams."""
    def get_ref_streets_stream(self, mode: str) -> TextIO:
        """Opens the reference street list of a relation."""
        path = self.get_ref_streets_path()
        return cast(TextIO, open(path, mode=mode))

    def __get_osm_streets_stream(self, mode: str) -> TextIO:
        """Opens the OSM street list of a relation."""
        path = self.get_osm_streets_path()
        return cast(TextIO, open(path, mode=mode))

    def get_osm_streets_csv_stream(self) -> util.CsvIO:
        """Gets a CSV reader for the OSM street list."""
        return util.CsvIO(self.__get_osm_streets_stream("r"))

    def __get_osm_housenumbers_stream(self, mode: str) -> TextIO:
        """Opens the OSM house number list of a relation."""
        path = self.get_osm_housenumbers_path()
        return cast(TextIO, open(path, mode=mode))

    def get_osm_housenumbers_csv_stream(self) -> util.CsvIO:
        """Gets a CSV reader for the OSM house number list."""
        return util.CsvIO(self.__get_osm_housenumbers_stream("r"))

    def get_ref_housenumbers_stream(self, mode: str) -> TextIO:
        """Opens the reference house number list of a relation."""
        return cast(TextIO, open(self.get_ref_housenumbers_path(), mode=mode))

    def get_housenumbers_percent_stream(self, mode: str) -> TextIO:
        """Opens the house number percent file of a relation."""
        return cast(TextIO, open(self.get_housenumbers_percent_path(), mode=mode))

    def get_housenumbers_htmlcache_stream(self, mode: str) -> TextIO:
        """Opens the house number HTML cache file of a relation."""
        return cast(TextIO, open(self.get_housenumbers_htmlcache_path(), mode=mode))

    def get_housenumbers_txtcache_stream(self, mode: str) -> TextIO:
        """Opens the house number plain text cache file of a relation."""
        return cast(TextIO, open(self.get_housenumbers_txtcache_path(), mode=mode))

    def get_streets_percent_stream(self, mode: str) -> TextIO:
        """Opens the street percent file of a relation."""
        return cast(TextIO, open(self.get_streets_percent_path(), mode=mode))

    def get_streets_additional_count_stream(self, mode: str) -> TextIO:
        """Opens the street additional count file of a relation."""
        return cast(TextIO, open(self.get_streets_additional_count_path(), mode=mode))

    def get_housenumbers_additional_count_stream(self, mode: str) -> TextIO:
        """Opens the housenumbers additional count file of a relation."""
        return cast(TextIO, open(self.get_housenumbers_additional_count_path(), mode=mode))

    def write_osm_streets(self, result: str) -> None:
        """Writes the result for overpass of Relation.get_osm_streets_query()."""
        with self.__get_osm_streets_stream("w") as sock:
            sock.write(result)

    def write_osm_housenumbers(self, result: str) -> None:
        """Writes the result for overpass of Relation.get_osm_housenumbers_query()."""
        with self.__get_osm_housenumbers_stream(mode="w") as stream:
            stream.write(result)

    def get_additional_housenumbers_htmlcache_stream(self, mode: str) -> TextIO:
        """Opens the additional house number HTML cache file of a relation."""
        return cast(TextIO, open(self.get_additional_housenumbers_htmlcache_path(), mode=mode))


# vim:set shiftwidth=4 softtabstop=4 expandtab:
