#!/usr/bin/env python3
#
# Version: 0.1.2
# Author: Chatbot Developers
# License: Apache License 2.0

"""
The accept_language module parses an Accept-Language HTTP header, originally from
<https://github.com/Babylonpartners/parse-accept-language>.
"""


from typing import List
import re

VALIDATE_LANG_REGEX = re.compile('^[a-z]+$', flags=re.IGNORECASE)
QUALITY_VAL_SUB_REGEX = re.compile('^q=', flags=re.IGNORECASE)
DEFAULT_QUALITY_VALUE = 1.0


class Lang:
    """One detected language."""
    def __init__(self, language: str, quality: float) -> None:
        self.__language = language
        self.__quality = quality

    def get_language(self) -> str:
        """Returns the language."""
        return self.__language

    def get_quality(self) -> float:
        """Returns the quality."""
        return self.__quality


def parse(accept_language_str: str) -> List[str]:
    """
    Parse a RFC 2616 Accept-Language string.
    https://www.w3.org/Protocols/rfc2616/rfc2616-sec14.html#sec14

    :param accept_language_str: A string in RFC 2616 format.
    """
    if not accept_language_str:
        return []

    parsed_langs = []
    for accept_lang_segment in accept_language_str.split(','):
        quality_value = DEFAULT_QUALITY_VALUE
        lang_code = accept_lang_segment.strip()
        if ';' in accept_lang_segment:
            lang_code, quality_value_string = accept_lang_segment.split(';')
            quality_value = float(QUALITY_VAL_SUB_REGEX.sub('', quality_value_string))

        lang_code_components = re.split('-|_', lang_code)
        if not all(VALIDATE_LANG_REGEX.match(c) for c in lang_code_components):
            continue

        if len(lang_code_components) == 1:
            # language code 2/3 letters, e.g. fr
            language = lang_code_components[0].lower()
        else:
            # full language tag, e.g. en-US
            language = lang_code_components[0].lower()
        parsed_langs.append(
            Lang(language=language, quality=quality_value)
        )
    return [i.get_language() for i in sorted(parsed_langs, key=lambda i: i.get_quality(), reverse=True)]

# vim:set shiftwidth=4 softtabstop=4 expandtab:
