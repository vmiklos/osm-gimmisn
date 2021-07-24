#!/usr/bin/env python3
#
# Copyright (c) 2021 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The area_files module contains file handling functionality, to be used by the areas module."""

import os
from typing import BinaryIO

import context
import i18n
import util


class RelationFilePaths:
    """A relation's file interface provides access to files associated with a relation."""
    def __init__(self, workdir: str, name: str):
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
    def get_ref_streets_read_stream(self, ctx: context.Context) -> BinaryIO:
        """Opens the reference street list of a relation for reading."""
        path = self.get_ref_streets_path()
        return ctx.get_file_system().open_read(path)

    def get_ref_streets_write_stream(self, ctx: context.Context) -> BinaryIO:
        """Opens the reference street list of a relation for wrtiting."""
        path = self.get_ref_streets_path()
        return ctx.get_file_system().open_write(path)

    def __get_osm_streets_read_stream(self, ctx: context.Context) -> BinaryIO:
        """Opens the OSM street list of a relation for reading."""
        path = self.get_osm_streets_path()
        return ctx.get_file_system().open_read(path)

    def __get_osm_streets_write_stream(self, ctx: context.Context) -> BinaryIO:
        """Opens the OSM street list of a relation for writing."""
        path = self.get_osm_streets_path()
        return ctx.get_file_system().open_write(path)

    def get_osm_streets_csv_stream(self, ctx: context.Context) -> util.CsvIO:
        """Gets a CSV reader for the OSM street list."""
        return util.CsvIO(self.__get_osm_streets_read_stream(ctx))

    def __get_osm_housenumbers_read_stream(self, ctx: context.Context) -> BinaryIO:
        """Opens the OSM house number list of a relation for reading."""
        path = self.get_osm_housenumbers_path()
        return ctx.get_file_system().open_read(path)

    def __get_osm_housenumbers_write_stream(self, ctx: context.Context) -> BinaryIO:
        """Opens the OSM house number list of a relation for writing."""
        path = self.get_osm_housenumbers_path()
        return ctx.get_file_system().open_write(path)

    def get_osm_housenumbers_csv_stream(self, ctx: context.Context) -> util.CsvIO:
        """Gets a CSV reader for the OSM house number list."""
        return util.CsvIO(self.__get_osm_housenumbers_read_stream(ctx))

    def get_ref_housenumbers_read_stream(self, ctx: context.Context) -> BinaryIO:
        """Opens the reference house number list of a relation for reading."""
        return ctx.get_file_system().open_read(self.get_ref_housenumbers_path())

    def get_ref_housenumbers_write_stream(self, ctx: context.Context) -> BinaryIO:
        """Opens the reference house number list of a relation for writing."""
        return ctx.get_file_system().open_write(self.get_ref_housenumbers_path())

    def get_housenumbers_percent_read_stream(self, ctx: context.Context) -> BinaryIO:
        """Opens the house number percent file of a relation for reading."""
        return ctx.get_file_system().open_read(self.get_housenumbers_percent_path())

    def get_housenumbers_percent_write_stream(self, ctx: context.Context) -> BinaryIO:
        """Opens the house number percent file of a relation for writing."""
        return ctx.get_file_system().open_write(self.get_housenumbers_percent_path())

    def get_housenumbers_htmlcache_read_stream(self, ctx: context.Context) -> BinaryIO:
        """Opens the house number HTML cache file of a relation for reading."""
        # The caller will do this:
        # pylint: disable=consider-using-with
        return ctx.get_file_system().open_read(self.get_housenumbers_htmlcache_path())

    def get_housenumbers_htmlcache_write_stream(self, ctx: context.Context) -> BinaryIO:
        """Opens the house number HTML cache file of a relation for writing."""
        # The caller will do this:
        # pylint: disable=consider-using-with
        return ctx.get_file_system().open_write(self.get_housenumbers_htmlcache_path())

    def get_housenumbers_txtcache_read_stream(self, ctx: context.Context) -> BinaryIO:
        """Opens the house number plain text cache file of a relation for reading."""
        # The caller will do this:
        # pylint: disable=consider-using-with
        return ctx.get_file_system().open_read(self.get_housenumbers_txtcache_path())

    def get_housenumbers_txtcache_write_stream(self, ctx: context.Context) -> BinaryIO:
        """Opens the house number plain text cache file of a relation for writing."""
        # The caller will do this:
        # pylint: disable=consider-using-with
        return ctx.get_file_system().open_write(self.get_housenumbers_txtcache_path())

    def get_streets_percent_read_stream(self, ctx: context.Context) -> BinaryIO:
        """Opens the street percent file of a relation for reading."""
        return ctx.get_file_system().open_read(self.get_streets_percent_path())

    def get_streets_percent_write_stream(self, ctx: context.Context) -> BinaryIO:
        """Opens the street percent file of a relation for writing."""
        return ctx.get_file_system().open_write(self.get_streets_percent_path())

    def get_streets_additional_count_read_stream(self, ctx: context.Context) -> BinaryIO:
        """Opens the street additional count file of a relation for reading."""
        return ctx.get_file_system().open_read(self.get_streets_additional_count_path())

    def get_streets_additional_count_write_stream(self, ctx: context.Context) -> BinaryIO:
        """Opens the street additional count file of a relation for writing."""
        return ctx.get_file_system().open_write(self.get_streets_additional_count_path())

    def get_housenumbers_additional_count_read_stream(self, ctx: context.Context) -> BinaryIO:
        """Opens the housenumbers additional count file of a relation for reading."""
        # The caller will do this:
        # pylint: disable=consider-using-with
        return ctx.get_file_system().open_read(self.get_housenumbers_additional_count_path())

    def get_housenumbers_additional_count_write_stream(self, ctx: context.Context) -> BinaryIO:
        """Opens the housenumbers additional count file of a relation for writing."""
        # The caller will do this:
        # pylint: disable=consider-using-with
        return ctx.get_file_system().open_write(self.get_housenumbers_additional_count_path())

    def write_osm_streets(self, ctx: context.Context, result: str) -> None:
        """Writes the result for overpass of Relation.get_osm_streets_query()."""
        with self.__get_osm_streets_write_stream(ctx) as sock:
            sock.write(util.to_bytes(result))

    def write_osm_housenumbers(self, ctx: context.Context, result: str) -> None:
        """Writes the result for overpass of Relation.get_osm_housenumbers_query()."""
        with self.__get_osm_housenumbers_write_stream(ctx) as stream:
            stream.write(util.to_bytes(result))

    def get_additional_housenumbers_htmlcache_read_stream(self, ctx: context.Context) -> BinaryIO:
        """Opens the additional house number HTML cache file of a relation for reading."""
        # The caller will do this:
        # pylint: disable=consider-using-with
        return ctx.get_file_system().open_read(self.get_additional_housenumbers_htmlcache_path())

    def get_additional_housenumbers_htmlcache_write_stream(self, ctx: context.Context) -> BinaryIO:
        """Opens the additional house number HTML cache file of a relation for writing."""
        # The caller will do this:
        # pylint: disable=consider-using-with
        return ctx.get_file_system().open_write(self.get_additional_housenumbers_htmlcache_path())


# vim:set shiftwidth=4 softtabstop=4 expandtab:
