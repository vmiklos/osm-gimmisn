#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

import re


def sort_numerically(strings):
    return sorted(strings, key=split_house_number)


def split_house_number(house_number):
    match = re.search(r"^([0-9]*)([^0-9].*|)$", house_number)
    number = 0
    try:
        number = int(match.group(1))
    except ValueError:
        pass
    return (number, match.group(2))


def sort_streets_csv(data):
    return process_csv_body(sort_streets, data)


def sort_streets(lines):
    return sorted(lines, key=split_street_line)


def split_street_line(line):
    field = line.split('\t')
    oid = get_array_nth(field, 0)
    name = get_array_nth(field, 1)
    highway = get_array_nth(field, 2)
    service = get_array_nth(field, 3)
    missing_name = name == ''
    return (missing_name, name, highway, service, oid)


def process_csv_body(fun, data):
    lines = data.split('\n')
    header = lines[0] if lines else ''
    body = lines[1:] if lines else ''
    result = [header] + fun(body)
    return '\n'.join(result)


def sort_housenumbers_csv(data):
    return process_csv_body(sort_housenumbers, data)


def sort_housenumbers(lines):
    return sorted(lines, key=split_housenumber_line)


def split_housenumber_line(line):
    field = line.split('\t')

    oid = get_array_nth(field, 0)
    street = get_array_nth(field, 1)
    housenumber = get_array_nth(field, 2)
    postcode = get_array_nth(field, 3)
    housename = get_array_nth(field, 4)
    cons = get_array_nth(field, 5)
    tail = field[6:] if len(field) > 6 else ''

    have_housenumber = housenumber != ''
    have_houseid = have_housenumber or housename != '' or cons != ''
    return (postcode, have_houseid, have_housenumber, street,
            split_house_number(housenumber),
            housename, split_house_number(cons), tail, oid)


def get_array_nth(arr, n):
    return arr[n] if len(arr) > n else ''


def simplify(s, spaceDecode=False):
    """ Handles normalization of a street name."""
    s = s.replace('Á', 'A').replace('á', 'a')
    s = s.replace('É', 'E').replace('é', 'e')
    s = s.replace('Í', 'I').replace('í', 'i')
    s = s.replace('Ó', 'O').replace('ó', 'o')
    s = s.replace('Ö', 'O').replace('ö', 'o')
    s = s.replace('Ő', 'O').replace('ő', 'o')
    s = s.replace('Ú', 'U').replace('ú', 'u')
    s = s.replace('Ü', 'U').replace('ü', 'u')
    s = s.replace('Ű', 'U').replace('ű', 'u')
    if spaceDecode:
        s = s.replace(' ', '%20')
    else:
        s = s.replace(' ', '_')
    s = s.lower()
    return s


def get_only_in_first(first, second):
    ret = []
    for i in first:
        if i not in second:
            ret.append(i)
    return ret


def get_in_both(first, second):
    ret = []
    for i in first:
        if i in second:
            ret.append(i)
    return ret
