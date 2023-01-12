# osm-gimmisn

[![tests](https://github.com/vmiklos/osm-gimmisn/workflows/tests/badge.svg)](https://github.com/vmiklos/osm-gimmisn/actions")

Finds objects missing from the OSM DB. As a start, it finds missing streets and house numbers based
on a reference list.

The latest version is v7.4, released on 2022-08-02.  See the
[release notes](https://github.com/vmiklos/osm-gimmisn/blob/master/NEWS.md).

## Description

It works by fetching the street and house number list for a relation (area), then suggesting what
possibly missing objects are a good idea to survey.

You can see this in action at public instances: [vmiklos.hu](https://osm-gimmisn.vmiklos.hu/osm),
[vasony.hu](https://osm.vasony.hu/).
