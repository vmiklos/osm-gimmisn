# Usage

## Up to date list of missing streets and house numbers

The [website](https://osm-gimmisn.vmiklos.hu/osm) provides you with hints regarding where to map
house numbers. The main page has the following columns:

- House number coverage: lists missing house numbers.

- Existing house numbers for an area: this is from the OSM database.

- Street coverage: lists missing streets.

- Existing streets for an area: this is from the OSM database.

- Area on the OSM website, you can see its boundary clearly there.

It is recommended to focus on the house number coverage, at least initially. If you see an
interesting area there, then you can find hints regarding what to survey. Consider the case when the
area is already in the list, the house number is indeed missing, and you just created an OSM change
to add it. The website is automatically updated in this case on a daily basis. You can refresh
manually if you want faster feedback:

- Wait a few minutes. If you just edited OSM, then running an overpass query right now will likely
  work with outdated data.

- Go to the missing house numbers section of your area, and click on the 'Update from OSM'
  link.

- Once the query is complete, the updated content should no longer mention your contributed house
  number(s) as missing anymore.

The missing house numbers are colored:

- black means a residential house number

- blue means a commercial house number (text view: `*` suffix)

- commercial house numbers can have comments, you can see them if you hover your mouse over them

NOTE: in case there is both a letter suffix and a source suffix, then the syntax is `42/A*`, i.e.
first the letter suffix, and then the source suffix.

## How to add a new area

A settlement, village or district of a large city is represented in the OSM database as relations.
Based on this, osm-gimmisn refers to managed areas as relations. To add a new relation, you need to
do the following steps:

- Be prepared to edit the [git repository](https://github.com/vmiklos/osm-gimmisn). This is possible
  via command-line `git clone` or via
  [web-based editing](https://help.github.com/en/articles/editing-files-in-your-repository).

- Search for the relation on [osm.org](https://www.openstreetmap.org), e.g. 'Kelenföld, Budapest'. The
  first hit is usually a relation link, e.g. https://www.openstreetmap.org/relation/2700869. Now you
  know the OSM identifier of the relation.

You'll also need the county reference ('refcounty' below) and settlement reference ('refsettlement'
below) of the area, you can find codes for Hungary in the
[refcodes](https://github.com/vmiklos/osm-gimmisn/blob/master/guide/refcodes) file.

(These codes are specific to Hungary, but in case your country has a larger and smaller container
for streets and house numbers, it's easy to adapt.)

- Add a new entry to the `data/relations.yaml` file in the git repository, using the following form:

```yaml
kelenfold:
    missing-streets: "no"
    osmrelation: 2700869
    refcounty: "01"
    refsettlement: "011"
```

Names are `snake_case` by convention, e.g. `kelenfold`. The other fields should match the values you
obtained in the previous bullet point. (`missing-streets: "no"` means that this OSM relation is
only a subset of the referenced settlement, so it's pointless to search for missing streets here.)

- Finally you can send your modification as a [pull
  request](https://github.com/vmiklos/osm-gimmisn/pull/new), it'll be probably accepted after
  review.

## Filtering (out) incorrect information

This action is similar to adding a new relation, but in this case you'll need to work with a file
dedicated to detailed information about the relation. The path derives from the relation name, e.g.
`relation-magasut.yaml`. Consider the case when you browse the Magasút (area) house number coverage
and you see a hint that the odd side of Magasúti köz (street) misses a house number: 999. Let's also
assume that you did your survey and you know that there is no such house number to be added to the
OSM database. The following steps area needed to silence this hint of osm-gimmisn:

- Survey the Magasúti köz street and note down which odd and even house number ranges are present
  for the street. Usually there are tables at the corners showing this information.

- Let's assume you found that the odd side is 1 to 9, the even side is 2 to 8. Now you just need to
  describe this, and osm-gimmisn will infer that 999 is noise in the reference data.

- Edit the `relation-magasut.yaml` file. If there is no such file, then the easiest is to copy an
  existing one and delete all streets.

- You can describe the Magasúti köz street like this:

```yaml
  Magasúti köz:
    ranges:
      - {start: '1', end: '9'}
      - {start: '2', end: '8'}
```

This is a machine-readable way of describing your survey result. In case the OSM and the reference
name of the street differs, use the OSM name here.

- Send a [pull request](https://github.com/vmiklos/osm-gimmisn/pull/new) to contribute your created
  filter, and then the website will be updated accordingly.

In other words, there is only minimal filtering for the reference data (1-999 and 2-998 is
accepted) by default. If you want to filter out noise, then you need to cover the correct house
numbers with ranges, and whatever is not in this range will be filtered out.

### Invalid house numbers instead of ranges

An alternative way of filtering out invalid data from the reference is explicitly stating what items
are invalid:

```yaml
  Magasúti köz:
    invalid: ['7', '11']
```

Sometimes this results in a more compact filter than the above presented `ranges` way. Note that the
values of the invalid list is compared to house numbers after normalization, e.g. '47/49D' can be
filtered out with '47'.

The items of the `invalid` list are normalized, the same way as reference house numbers. So in
case '42a' is normalized to '42', you can write '42a' in the invalid list, and it'll silence '42'.
This has the benefit that the `invalid` items will keep working even if you later decide to set the
`housenumber-letters: true` mode.

The items of the `invalid` list containing hyphens (`-`) are handled before other `invalid` items:
the hyphen is not expanded, and they filter out reference items. Both sides delete slashes (`/`) and
convert to lowercase before comparing.

## Searching for missing streets

The yaml format is like this:

- `missing-streets: only`: this is provided in the root of the `relation-NAME.yaml` file (by
  convention). It denotes that you only want to search for streets in this (potentially large)
  relation. Other valid values are `yes` (which is the default) and `no`.

For each reported missing street, the outcomes can be the followings:

- add the missing street to OSM, based on survey

- add a mapping between the reference and OSM name if survey confirms that the name in OSM is
  correct (`refstreets` key)

- silence the street name if it should have no equivalent in OSM (`street-filters` key)

## Searching for additional streets

The purpose of this check is to detect street names in OSM, which are not in the reference, i.e.
"additional" is the opposite of "missing".

For each reported additional street, the outcomes can be the followings:

- fix the name of the additional street in OSM, based on survey

- add a mapping between the reference and OSM name if survey confirms that the name in OSM is
  correct (`refstreets` key)

- silence the street name if it should have no equivalent in the reference (`osm-street-filters` key)

## Advanced topics

Apart from filtering out noise, you can also specify other settings, though these are needed less
frequently:

- `refstreets`: this key can be used in the root of a relation file, it's used to describe street
  name mappings, in case the OSM name and reference name differs and the OSM one is the correct
  name. The key is the OSM name and the value is the reference name. It's not valid to map multiple
  OSM names to the same reference name, so this has to be a 1:1 mapping. This makes it possible to
  map both ways using the same markup.

- `street-filters`: this key can be used in the root of a relation file, it's used to silence false
  alarms during the 'missing streets' check when a reference street name should have no OSM street
  name equivalent.

- `osm-street-filters`: this key can be used in the root of a relation file, it's used to silence false
  alarms during the 'additional streets' check when an OSM street name should have no reference
  street name equivalent.

- `refsettlement`: this key can be used for a street. In case the majority of a relation has a given
  `refsettlement` value, but there are a few exceptions, then you can use this markup to override the
  relation-level value with a street-level one.

- Range-level `refsettlement`: this is useful in case the two sides of a street has different
  `refsettlement` values (that side of the street belongs to a different district or settlement).

- `interpolation`: this key can be specified for a street. Its `all` value means that the street has
  continuous numbering instead of even and odd sides.

- `show-refstreet: false`: this key can be specified for a street. It means that in case the OSM and
  reference names would not match, don't show the reference name on the missing housenumbers -> view
  results page.

NOTE: This has a second effect as well. The `/missing-streets/.../view-turbo` page lists all OSM
street names which have a mapping to reference names. Before presenting that list, items with this
`show-refstreet: false` property are filtered out from the result. This supports a workflow where
the mapping has guesses as well, and then survey clarifies those questionable items, so that either
OSM is fixed or `show-refstreet: false` is added.

- `inactive: true`: this key can be used for a relation, it disables the daily update (which would
  be a waste if e.g. the relation already has 100% coverage.) Manual updates are still possible.

- You can download a GPX file showing the streets of the missing house numbers if you follow the
  'Overpass turbo query for the below streets' link on the missing housenumbers page. To do this,
  visit the 'Overpass turbo' site from the toolbar, copy the query, run it, choose Export -> Download
  as GPX, and e.g. load the result into OsmAnd on your phone.

- `housenumber-letters: true`: this key can be used to do micro-mapping, i.e. detect that e.g. 42/B
  is missing, even if 42/A is already mapped. Works with 42/2 and 42/1 as well. (The default
  behavior is to ignore any noise after the numeric value of the house numbers.)

- `alias: ["foo", "bar"]`: this key can be used on relations to specify old names. This way
  bookmarks keep working, even in case a relation is renamed.

- `additional-housenumbers: true`: this key can be used to opt-in to see house numbers which are on
  OSM but not in the reference. It's disabled by default as it may lead to unwanted vandalism. See
  below for details.

It is expected that "normalization" not only filters out noise from the reference, but also expands
housenumber ranges in a sensible way. Here are some examples:

|Case ID|Given a range|When this setting is used             |Expands to                                           |
|-------|-------------|--------------------------------------|-----------------------------------------------------|
|1      |139          |range is `{start: '137', end: '165'}` |139 as it is in range                                |
|2      |999          |range is `{start: '137', end: '165'}` |Empty list as it is not in range                     |
|3      |x            |Defaults                              |Empty list as it is not a number                     |
|4      |1            |Defaults                              |1, as the default ranges are 1-999 and 2-998         |
|5      |1;2          |Defaults                              |1 and 2 as a semicolon is a separator                |
|6      |2-6          |Defaults                              |2, 4, and 6 as the even range is expanded            |
|7      |5-8          |Defaults                              |5 and 8 as the parity doesn't match                  |
|8      |2-5          |`interpolation=all`                   |2, 3, 4 and 5                                        |
|9      |163-167      |range is `{start: '137', end: '165'}` |163 and 165, no 167                                  |
|10     |2-2000       |Defaults                              |2 because 2000 large(r than 1000)                    |
|11     |2-56         |Defaults                              |2 and 56 because the diff of two is large(r than 24) |
|12     |0-42         |Defaults                              |42 because 0 is too small                            |
|13     |42-1         |Defaults                              |42 because -1 is considered as a suffix              |

See the tests in `src/areas/tests.rs` for even more details.

### Additional house numbers analysis

This is the opposite of missing housenumbers, i.e. check for OSM objects which are not in the
reference. This can be helpful to find errors, but use it with care: just because you did not find a
housenumber by survey, it doesn't mean it has to be deleted.

The way to filter out valid data from the OSM list is to explicitly state what items are valid:

```yaml
  Magasúti köz:
    valid: ['13', '15']
```

### Automerge workflow for committers

If you contribute to osm-gimmisn frequently, then you'll likely get self-review permissions granted.
Once that's the case you can use this workflow to submit your changes in a fire & forget way, from
command-line:

- Once:
  [set up your ssh key](https://docs.github.com/en/free-pro-team@latest/github/authenticating-to-github/adding-a-new-ssh-key-to-your-github-account)

- Once: `git clone git@github.com:vmiklos/osm-gimmisn`

- Once: `cd osm-gimmisn`

- For each PR: this can be repeated in case you want multiple commits in a single PR or CI finds an
  error:

```console
git fetch --prune # if the PR has been merged on the server, then the remote private/$USER/master has been deleted, we learn about that here
git rebase origin/master # we work on a fresh master
... hack hack hack ...
git commit -a -m "data: blabla"
git show # optional but recommended: review your changes, "q" quits from less
git push origin master:private/$USER/master
```

- After this, open the PR [from your browser](https://github.com/vmiklos/osm-gimmisn/pull/new/private/$USER/master)

- Agree to create the PR, finally push the
  [Enable
  auto-merge](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/incorporating-changes-from-a-pull-request/automatically-merging-a-pull-request)
  button.

### Developer API

In case the `/missing-housenumbers/.../view-result` HTML output looks interesting to you and you
would like to use that information in your application, no need to scrape the webpage, you can get
the raw input of that analysis as `/missing-housenumbers/.../view-result.json` instead.

Similarly, the `/additional-housenumbers/.../view-result` HTML output has a matching
`/additional-housenumbers/.../view-result.json`.
