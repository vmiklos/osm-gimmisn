# Changelog

## master

- Resolves: gh#3744 cron: add a new --refarea switch
- Resolves: gh#3558 `,` is now also recognized as a housenumber separator, similar to `;`
- Resolves: gh#3768 lints, invalid addr:city values: add new 'update from OSM' button
- Resolves: gh#3792 /lints/whole-country/invalid-addr-cities now has separate osm and areas dates
- Resolves: gh#3818 invalid addr:city values: make sure history is not modified after the fact
- Resolves: gh#3826 sync-ref has a new `--mode local` switch to work offline
- Resolves: gh#3850 validator: flag osm=ref in refstreets

## 24.2

- Resolves: gh#3073 New `/missing-housenumbers/.../view-lints` endpoint, listing per-relation lints
  (mostly flagging unused filters)
- Resolves: gh#3288 additional streets, gpx: handle nodes and relations as well
- Resolves: gh#3290 YAML keys are now flagged by the validator, instead of taking the last value
- Resolves: gh#3456 `/lints/whole-country/invalid-addr-cities` is now case-sensitive, finds more
  problems
- Resolves: gh#3105 update times in the footer now show both the OSM and areas timestamp instead of
  the time of the overpass query

## 7.6

- Rouille: new `--host` parameter to specify the bind address
- The `/missing-housenumbers/.../update-result` is now about 6 times faster (replaced home-grown
  JSON cache with SQL indexes)
- New `/lints/.../invalid-addr-cities` endpoint, tries to find invalid addr:city values
- Resolves: gh#2986 stats: the length of the invalid addr:city values list now has a chart
- Resolves: gh#2987 stats: extract 2 lints from the stats page to an own lints page
- Resolves: gh#2994 areas: find ref-not-in-reflist problems in `Relation.get_invalid_refstreets()`
- Resolves: gh#2988 cron: enable inactive relations which are invalid
- Resolves: gh#3018 additional streets is now available in gpx format as well

## 7.5

- New `/missing-housenumbers/.../view-result.json` endpoint, exposing the missing-housenumbers
  analysis result for a relation in a machine-readable format.
- New `/additional-housenumbers/.../view-result.json` endpoint, exposing the additional-housenumbers
  analysis result for a relation in a machine-readable format.
- Resolves: gh#2592 cron now creates state for new, inactive relations
- Resolves: gh#2628 rename `*.expected-data` to `*.overpassql`

## 7.4

- Ported to chartjs v3, the javascript bundle size is now about 20% smaller

## 7.3

- Ported to Rust, the missing-housenumbers analysis is now about 5 times faster
- Resolves: gh#1664 re-try overpass when response is XML (an error message) and CSV was requested
- Resolves: gh#1740 fix /filter-for/refcounty/../refsettlement/.. filtering out everything
- Resolves: gh#1746 fix disappeared localized strings
- Resolves: gh#1815 validator now rejects trailing whitespace when converting numbers to strings
- Resolves: gh#1839 stats: report per-zip coverage of housenumbers
- Resolves: gh#1950 fix the validator's exit code to actually fail on validation errors during CI
- Resolves: gh#2009 stats: separate progressbar for the capital

## 7.2

- Resolves: gh#964 `get_street_from_housenumber`: consider addr:place when addr:street is empty

- Resolves: gh#978 gettext: check `wsgi_additional.py` for l10n strings

- Resolves: gh#1008 invalid refstreets: add to missing/additional streets as well

- Resolves: gh#1025 invalid refstreets: add UI to list all problematic relations

- Resolves: gh#1033 missing streets: let "update from osm" also update house numbers

- Resolves: gh#1048 invalid-relations: flag filter key names which are not OSM street names

- Resolves: gh#1067 invalid refstreets: don't show note when there are no results

- Resolves: gh#1126 stats, invalid-relations: ignore relation without an osm street list

- Resolves: gh#1198 tests: fix missing `test_config.TestCase` base class

- Resolves: gh#1206 Cache missing-housenumbers plain text output

- Resolves: gh#1201 `parse_access_log`: ignore complete areas harder

- Resolves: gh#1209 cron: also cache localized missing housenumbers html output

- Resolves: gh#1225 missing-housenumbers html cache: handle missing relation yaml

- Resolves: gh#1408 Fix Hungarian sort order

- Resolves: gh#1463 stats: handle missing files in `get_topcities()`

- Resolves: gh#1508 webframe: context time is in utc, datetime.date.fromtimestamp takes a local time

## 7.1

- Resolves: gh#576 garbage collection: if a relation was created <1 month ago, then ignore it in
  `parse_access_log.py`

- Resolves: gh#578 garbage collection: exclude completed areas

- Resolves: gh#585 fix street mapping when street name contains commas

- Resolves: gh#594 stats: top cities: filter for valid cities + captial districts

- Resolves: gh#611 additional streets, opposite of missing streets are now searched for

- Resolves: gh#613 provide a robots.txt

- Resolves: gh#616 handle house numbers containing commas

- Resolves: gh#617 garbage collection: filter out search engine user agents

- Resolves: gh#688 reworked the toolbar

- Resolves: gh#689 additional streets: improved main page, showing # of additional streets there

- Resolves: gh#690 you can now filter for relations based on your current position

- Resolves: gh#754 missing housenumbers now ignores points without house numbers and
  conscriptionnumbers

- Resolves: gh#759 main page: default to hiding complete areas

- Resolves: gh#765 calls to overpass now always redirect back to the original page

- Resolves: gh#782 additional streets: handle streets with type=relation

- Resolves: gh#784 street mappings: invalid osm or reference names are now flagged on the missing
  housenumbers page

- Resolves: gh#830 additional streets: track OSM id of street names which come from house numbers

- Resolves: gh#861 additional streets now can generate an overpass query

- Resolves: gh#875 missing and additional streets now also have plain text and checklist output
  formats

- Resolves: gh#886 additional streets have a proper last update date

## 7.0

- Resolves: gh#564 wsgi: add support for locale-aware percent formatting

- Resolves: gh#563 city progress: avoid double link

- Resolves: gh#561 cron: update ref.count from `reference_citycounts`

- Resolves: gh#521 sync missing-housenumbers and missing-streets toolbars

- Resolves: gh#513 add overpass query for not yet audited street mappings

- Resolves: gh#490 stats: add per-city coverage page

- Resolves: gh#489 stats: add per-city monthly diff page

- The stats feature now tracks the count of house number editors over time

## 6.0

- The stats feature now has localized strings

- The stats feature now has good test coverage

- Resolves: gh#441 cron can now again update inactive relations once a month

- Resolves: gh#437 the locale install requirement is now documented

- Resolves: gh#436 ref street name is now shown after the osm street name if it differs

- Resolves: gh#381 stats now has trendlines

- Resolves: gh#426 missing housenumbers now allows updating from OSM with a single click

- Resolves: gh#383 stats, monthly total now has an extra column to show today's count

- Resolves: gh#385 `invalid` list items are now normalized by default

- Resolves: gh#388 cron's console output and log file now uses the same format

- Resolves: gh#414 the missing housenumber page's table is now sorted correctly even if house number
  ranges reduce the amount of items for some streets

- Resolves: gh#372 commercial house numbers now can have comments, visible as tooltips

## 5.0

- A new `/osm/housenumber-stats/whole-country/` page featuring new and all-time house number data

- A new `cherry.py` glue layer to help running on top of CherryPy

- gh#380 the validator now catches strings which are
  not valid items in an `invalid:` string list

- gh#363 next to letter suffixes (42/a), now digit
  suffixes are also accepted (42/1). Both are still limited to a single-char suffix (2020-03-23)

## 4.0

- gh#344 next to the existing "txt" output, a new
  "chkl" output is available for missing house numbers of a relation, providing a plain text
  checklist. (2020-03-07)

- yaml files are now parsed build-time to improve performance (main page loads 7 times faster)

- complete line coverage for the cron code, which was the last uncovered module

## 3.0

- complete test coverage for the wsgi code

## 2.0

- gh#322 alias names are now supported for relations,
  so compatibility (with existing bookmarks) does not break when renaming. (2019-01-10)

- gh#291 added error handling for not valid relation
  names. (2019-12-12)

- gh#285: HTML output uses 42/A style for
  letter-suffixed house numbers, but plain text output uses 42a to help turning the output into
  `invalid` configs. (2019-12-06)

- gh#267: it is now possible to opt in for a more
  strict behavior where 42/B is not considered mapped when 42/A is already mapped. (2019-11-29)

- gh#269: noise in the reference can be now cleaned by
  filtering out house numbers explicitly, rather than filtering for valid ranges. (2019-11-15)

- gh#195: track what source range generated what house
  numbers for more compact results. (2019-11-10)

- gh#224: a way to generate the gpx of all streets
  missing house numbers. (2019-10-31)

- gh#237: make OSM IDs of existing house numbers
  clickable. (2019-10-22)

- gh#228: added time internal hint when the overpass
  query errors out due to not waiting enough. (2019-10-12)

- gh#204: added possibility to list certain
  reftelepules names when a specific refmegye is selected on the main page. (2019-10-09)

## 1.0

- Initial release

Enhancements up to 2019-10-07 were presented at <https://www.meetup.com/OpenStreetMap-Hungary/>.
