# Introduction

osm-gimmisn is a web-based tool to find objects missing from the OpenStreetMap database. As a start,
it finds missing streets and house numbers based on a reference list.

The latest version is v24.2, released on 2024-02-01. See the [release notes](news.md).

It works by fetching the street and house number list for a relation (area), then suggesting what
possibly missing objects are a good idea to survey.

You can see this in action at public instances: [vmiklos.hu](https://osm-gimmisn.vmiklos.hu/),
[vasony.hu](https://osm.vasony.hu/).

## Website

Check out the [project's website](https://vmiklos.hu/osm-gimmisn/) for a list of features and
installation and usage information.

## Platforms

osm-gimmisn has been used on Linux, but the data validator is known to work Windows as well.

## The important bits of the code

- The entry point is the `application()` function in `src/wsgi.rs`.

- The test code lives under `src/*/test.rs`.

- The documentation is under `guide/`.

## License

Use of this source code is governed by a BSD-style license that can be found in the LICENSE file.
See the [license bundle](license.html) for details.
