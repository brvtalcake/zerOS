#!/usr/bin/env python3

import os
import unicodedata
import inspect
import re
import mpire as mp
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
    return unicodedata.category(c).startswith('Z') or unicodedata.category(c).startswith('C')

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

def _get_unicode_cat_as_dict(catpred: Callable[[str], bool]) -> dict[int, bool]:
    return {c: catpred(chr(c)) for c in range(0x10FFFF)}

ALNUM_DICT: dict[int, bool]
DIGIT_DICT: dict[int, bool]
ALPHA_DICT: dict[int, bool]
WHITESPACE_DICT: dict[int, bool]
PUNCT_DICT: dict[int, bool]
SYMBOL_DICT: dict[int, bool]

ALNUM_REGEX: re.Pattern[str] | None = None
DIGIT_REGEX: re.Pattern[str] | None = None
ALPHA_REGEX: re.Pattern[str] | None = None
WHITESPACE_REGEX: re.Pattern[str] | None = None
PUNCT_REGEX: re.Pattern[str] | None = None
SYMBOL_REGEX: re.Pattern[str] | None = None

ESCAPED_DICT = {}

def __fill_escaped_parallel():
    global ESCAPED_DICT
    with mp.WorkerPool(n_jobs=6, daemon=False, use_dill=True, enable_insights=True, start_method='spawn') as p:
        results = p.map(
            _get_unicode_cat_list, [
                _isalnum,
                _isdigit,
                _isalpha,
                _iswhitespace,
                _ispunct,
                _issymbol
            ]
        )
        assert isinstance(results, list) and all([isinstance(x, str) for x in results])
        ESCAPED_DICT['ALNUM'], ESCAPED_DICT['DIGIT'], ESCAPED_DICT['ALPHA'], ESCAPED_DICT['WHITESPACE'], ESCAPED_DICT['PUNCT'], ESCAPED_DICT['SYMBOL'] = results
    return None

def fill_regex_parallel():
    global ALNUM_REGEX
    global DIGIT_REGEX
    global ALPHA_REGEX
    global WHITESPACE_REGEX
    global PUNCT_REGEX
    global SYMBOL_REGEX
    __fill_escaped_parallel()
    ALNUM_REGEX = re.compile(f'[{ESCAPED_DICT["ALNUM"]}]', re.UNICODE)
    DIGIT_REGEX = re.compile(f'[{ESCAPED_DICT["DIGIT"]}]', re.UNICODE)
    ALPHA_REGEX = re.compile(f'[{ESCAPED_DICT["ALPHA"]}]', re.UNICODE)
    WHITESPACE_REGEX = re.compile(f'[{ESCAPED_DICT["WHITESPACE"]}]', re.UNICODE)
    PUNCT_REGEX = re.compile(f'[{ESCAPED_DICT["PUNCT"]}]', re.UNICODE)
    SYMBOL_REGEX = re.compile(f'[{ESCAPED_DICT["SYMBOL"]}]', re.UNICODE)
    return None

from mpire.async_result import AsyncResult

class DictWrapper(object):
    def __init__(self, res: AsyncResult) -> None:
        self.result = res
        self.value = None
        return None
    def __call__(self, to: float | None = None) -> dict[int, bool]:
        return self._get(to)
    def _get(self, t):
        if self.value is not None:
            return self.value
        if not self.result.ready():
            self.result.wait(t)
        if not self.result.successful():
            raise RuntimeError('Error while parallel computing of dict[int, bool] in module ' + __name__)
        self.value = self.result.get(0.00001)
        return self.value

def get_dicts_parallel(p: mp.WorkerPool) -> tuple[DictWrapper, ...]:
    def get_wrappers(*args: AsyncResult):
        return tuple(map(DictWrapper, args))
    def apply(catpreds: list[Callable[[str], bool]]) -> tuple[AsyncResult, ...]:
        def do_apply(catpred: Callable[[str], bool]) -> AsyncResult:
            return p.apply_async(_get_unicode_cat_as_dict, catpred)
        return tuple(
            map(do_apply, catpreds)
        )
    return get_wrappers(
        *apply([_isalnum, _isdigit, _isalpha, _iswhitespace, _ispunct, _issymbol])
    )

if __name__ in ['__main__']:
    __fill_escaped_parallel()
    #ALNUM_LIST = _get_unicode_cat_as_list(_isalnum)
    #DIGIT_LIST = _get_unicode_cat_as_list(_isdigit)
    #ALPHA_LIST = _get_unicode_cat_as_list(_isalpha)
    #WHITESPACE_LIST = _get_unicode_cat_as_list(_iswhitespace)
    #PUNCT_LIST = _get_unicode_cat_as_list(_ispunct)
    #SYMBOL_LIST = _get_unicode_cat_as_list(_issymbol)

    ALNUM_REGEX = re.compile(f'[{ESCAPED_DICT["ALNUM"]}]', re.UNICODE)
    DIGIT_REGEX = re.compile(f'[{ESCAPED_DICT["DIGIT"]}]', re.UNICODE)
    ALPHA_REGEX = re.compile(f'[{ESCAPED_DICT["ALPHA"]}]', re.UNICODE)
    WHITESPACE_REGEX = re.compile(f'[{ESCAPED_DICT["WHITESPACE"]}]', re.UNICODE)
    PUNCT_REGEX = re.compile(f'[{ESCAPED_DICT["PUNCT"]}]', re.UNICODE)
    SYMBOL_REGEX = re.compile(f'[{ESCAPED_DICT["SYMBOL"]}]', re.UNICODE)

def __fill_dicts_parallel():
    global ALNUM_DICT, DIGIT_DICT, ALPHA_DICT, WHITESPACE_DICT, PUNCT_DICT, SYMBOL_DICT
    with mp.WorkerPool(n_jobs=6, daemon=False, use_dill=True, enable_insights=True, start_method='spawn') as pool:
        ALNUM_DICT_async = pool.apply_async(_get_unicode_cat_as_dict, (_isalnum,))
        DIGIT_DICT_async = pool.apply_async(_get_unicode_cat_as_dict, (_isdigit,))
        ALPHA_DICT_async = pool.apply_async(_get_unicode_cat_as_dict, (_isalpha,))
        WHITESPACE_DICT_async = pool.apply_async(_get_unicode_cat_as_dict, (_iswhitespace,))
        PUNCT_DICT_async = pool.apply_async(_get_unicode_cat_as_dict, (_ispunct,))
        SYMBOL_DICT_async = pool.apply_async(_get_unicode_cat_as_dict, (_issymbol,))
        while True:
            ready_list = [x.ready() for x in [ALNUM_DICT_async, DIGIT_DICT_async, ALPHA_DICT_async, WHITESPACE_DICT_async, PUNCT_DICT_async, SYMBOL_DICT_async]]
            success_list = [x.successful() for x in [ALNUM_DICT_async, DIGIT_DICT_async, ALPHA_DICT_async, WHITESPACE_DICT_async, PUNCT_DICT_async, SYMBOL_DICT_async]]
            if all(ready_list):
                if all(success_list):
                    ALNUM_DICT = ALNUM_DICT_async.get()
                    DIGIT_DICT = DIGIT_DICT_async.get()
                    ALPHA_DICT = ALPHA_DICT_async.get()
                    WHITESPACE_DICT = WHITESPACE_DICT_async.get()
                    PUNCT_DICT = PUNCT_DICT_async.get()
                    SYMBOL_DICT = SYMBOL_DICT_async.get()
                    break
                else:
                    raise Exception('Some process failed')
    return None

def _get_exe_dir() -> str:
    return os.path.dirname(os.path.realpath(__file__))

def _write_lark_grammar():
    TERM_TEMPLATE = Template('$term: /[$escaped]/u')
    with open(os.path.join(_get_exe_dir(), 'escaped_unicode.lark'), 'w') as f:
        for k, v in ESCAPED_DICT.items():
            f.write(f'{TERM_TEMPLATE.substitute(term=k, escaped=v)}\n')
        f.close()
    return None
_write_lark_grammar()
