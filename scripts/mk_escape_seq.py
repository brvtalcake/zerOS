#!/usr/bin/env python3

import os
import unicodedata
import inspect
from typing import Callable
from string import Template

def _get_rettype(func):
    return inspect.signature(func).return_annotation

def _isalpha(c: str) -> bool:
    return unicodedata.category(c).startswith('L')

def _isdigit(c: str) -> bool:
    return unicodedata.category(c).startswith('N')

def _isalnum(c: str) -> bool:
    return _isalpha(c) or _isdigit(c)

def _iswhitespace(c: str) -> bool:
    return unicodedata.category(c).startswith('Z')

def _ispunct(c: str) -> bool:
    return unicodedata.category(c).startswith('P')

def _issymbol(c: str) -> bool:
    return unicodedata.category(c).startswith('S')

def _get_unicode_cat_ranges(catpred: Callable[[str], bool]) -> list[tuple[int, int]]:
    ret: list[tuple[int, int]] = []
    start = -1
    end = -1
    for i in range(0x10FFFF):
        if catpred(chr(i)):
            if start == -1:
                start = i
            end = i
        else:
            if start != -1:
                ret.append((start, end))
                start = -1
                end = -1
    if start != -1:
        ret.append((start, end))
    return ret

def escape_unicode(c: str) -> str:
    return f'\\u{ord(c):04x}'

def _get_unicode_alnums_list2(escape=True) -> str:
    def _maybe_escape(x):
        if escape:
            return escape_unicode(chr(x))
        return chr(x)
    return ''.join([_maybe_escape(x) for t in _get_unicode_cat_ranges(_isalnum) for x in range(t[0], t[1] + 1)])

def _get_unicode_cat_list(catpred: Callable[[str], bool], escape=True) -> str:
    def _maybe_escape(x):
        if escape:
            return escape_unicode(chr(x))
        return chr(x)
    ret: str = ''
    for c in range(0x10FFFF):
        if catpred(chr(c)):
            ret += _maybe_escape(c)
    return ret

def _get_unicode_cat_as_list(catpred: Callable[[str], bool]) -> list[int]:
    return [c for c in range(0x10FFFF) if catpred(chr(c))]

ESCAPED_DICT = {
    'ALNUM': _get_unicode_cat_list(_isalnum),
    'DIGIT': _get_unicode_cat_list(_isdigit),
    'ALPHA': _get_unicode_cat_list(_isalpha),
    'WHITESPACE': _get_unicode_cat_list(_iswhitespace),
    'PUNCT': _get_unicode_cat_list(_ispunct),
    'SYMBOL': _get_unicode_cat_list(_issymbol)
}

ALNUM_LIST = _get_unicode_cat_as_list(_isalnum)
DIGIT_LIST = _get_unicode_cat_as_list(_isdigit)
ALPHA_LIST = _get_unicode_cat_as_list(_isalpha)
WHITESPACE_LIST = _get_unicode_cat_as_list(_iswhitespace)
PUNCT_LIST = _get_unicode_cat_as_list(_ispunct)
SYMBOL_LIST = _get_unicode_cat_as_list(_issymbol)

def _get_exe_dir() -> str:
    return os.path.dirname(os.path.realpath(__file__))

def _write_lark_grammar():
    TERM_TEMPLATE = Template('$term: /[$escaped]/u')
    with open(os.path.join(_get_exe_dir(), 'escaped_unicode.lark'), 'w') as f:
        for k, v in ESCAPED_DICT.items():
            f.write(f'{TERM_TEMPLATE.substitute(term=k, escaped=v)}\n')
        f.close()
    return

if __name__ == '__main__':
    _write_lark_grammar()
