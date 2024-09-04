#!/usr/bin/env python3

import typing
from typing import Any, TypeVar, Iterable, Literal, Optional
from dl_intel_manual import do_dl

import pymupdf
import pandas
import re

_K = TypeVar("_K")
_V = TypeVar("_V")

def _page(xs: Iterable[Any], n: int) -> Any:
    for x in xs:
        if x.number == n:
            return x
    return None

class SafeDict(dict[_K, _V]):
    def safe_update(self, other: dict[_K, _V], throw=True):
        for k, v in other.items():
            tmp = self.setdefault(k, v)
            if tmp != v:
                if throw:
                    raise RuntimeError(f"Trying to add value {v} for already existing key {k} with value {tmp}")
        return self

def _intel_extract_caption(page: Any, table_bbox: pymupdf.Rect) -> str:
    h = 20
    upper_bbox = pymupdf.Rect(
        table_bbox.x0,
        table_bbox.y0 - h,
        table_bbox.x1,
        table_bbox.y0
    )
    res = page.get_textbox(upper_bbox).strip()
    if res is None or len(res) <= 0:
        raise RuntimeError("Error while trying to extract caption")
    return res

def _intel_extract_table_with_name(pages: list[Any], number: str, contains: str | None = None) -> list[pandas.DataFrame]:
    ret = []
    table_regex = f'Table {number.strip()}' + r'\.'
    if contains is not None:
        table_regex += r'.*(' + contains + r').*'
    else:
        table_regex += r'.+'
    for page in pages:
        finder: pymupdf.table.TableFinder = page.find_tables()
        captions = [_intel_extract_caption(page, pymupdf.Rect(*(table.bbox))) for table in finder.tables]
        for tab, cap in [(finder.tables[i], captions[i]) for i in range(len(captions))]:
            if re.match(table_regex, cap) is not None:
                ret.append(tab.to_pandas())
    return ret

def _intel_isolate_volume4(doc: Any) -> list[Any]:
    toc = doc.get_toc(simple=False)
    lvl2chapters = [chap for chap in toc if chap[0] == 2]
    vol4chap: int = -1
    for i, chap in enumerate(lvl2chapters):
        if chap[1] == 'Volume 4: Model-Specific Registers':
            vol4chap = i
            break
    if vol4chap == -1:
        raise RuntimeError("Unable to find Intel-SDM Volume 4")
    x = lvl2chapters[vol4chap][3]['page']
    y = lvl2chapters[vol4chap + 1][3]['page']
    vol4pages = [page for page in doc[x:y]]
    return vol4pages

def _intel_get_alderlake_msrs(doc: Any) -> dict[str, int]:
    vol4 = _intel_isolate_volume4(doc)
    return { }

def _intel_join_and_clean_table_2_2(fragments: list[pymupdf.table.Table]) -> list[list[str]]:
    breakpoint()
    ret: list[list[str]] = []
    hexchar = r'[0-9A-Fa-f]'
    for frag in fragments:
        rows: list[list[str]] = []
        uncurrated: list[list[str | None]] = frag.extract()
        uncurrated = uncurrated[2:] # Skip fields description
        for i in range(len(uncurrated)):
            new = [elem.strip() for elem in uncurrated[i] if elem is not None]
            uncurrated[i] = typing.cast(list[str | None], new)
        def _line_matches(n: int, pattern: tuple[re.Pattern[str], ...]) -> Optional[tuple[re.Match[str], ...]]:
            line: list[str] = typing.cast(list[str], uncurrated[n])
            if len(line) != len(pattern):
                return None
            results = tuple(map(lambda t: t[0].fullmatch(t[1]), zip(pattern, line)))
            if results.count(None) != 0:
                return None
            return typing.cast(tuple[re.Match[str], ...], results)
        def _get_next_reg(start: int) -> tuple[str, str, str, int] | None:
            if start >= len(uncurrated):
                return None
            unformated_regaddr_pattern = r'Register Address: {}+H, (\d+)'
            regaddr_regname_pattern: tuple[re.Pattern[str], re.Pattern[str]] = (
                re.compile(
                    unformated_regaddr_pattern.format(hexchar)
                ),
                re.compile(
                    r'(\w+) .*'
                )
            )
            regaddr: str | None = None
            regname: str | None = None
            regintroduction: str | None = None
            next   : int | None = None
            for i in range(start, len(uncurrated)):
                if regaddr is None:
                    assert regname is None
                    match = _line_matches(i, regaddr_regname_pattern)
                    if match is not None:
                        print("MATCHED")
                        for m in match:
                            print(m.groups())
                        exit()
        while _get_next_reg(0) is not None: pass
        ret.extend(rows)
        


def _intel_join_and_clean_table(nb: str, fragments: list[pymupdf.table.Table]) -> list[list[str]]:
    return eval('_intel_join_and_clean_table_' + nb.strip().replace('-', '_'))(fragments)

def _intel_get_architectural_msrs(doc: Any) -> dict[str, int]:
    vol4 = _intel_isolate_volume4(doc)
    fragments = _intel_extract_table_with_name(vol4, '2-2', 'IA-32 Architectural MSRs')
    for frag in fragments:
        print(frag.extract())
    exit()
    table = _intel_join_and_clean_table('2-2', fragments)
    return { }

def intel_extract_msrs(mach: str = "alderlake") -> SafeDict[str, int]:
    pdf: bytes = do_dl()
    msrs: SafeDict[str, int] = SafeDict()
    archmsrs: dict[str, int] = { }
    machmsrs: dict[str, int] = { }
    with pymupdf.open(stream=pdf, filetype='pdf') as pdfio:
        archmsrs = _intel_get_architectural_msrs(pdfio)
        try:
            machmsrs = eval("_intel_get_" + mach + "_msrs")(pdfio)
        except:
            raise RuntimeError(f"MSR extraction for intel processor `{mach}` is not implemented")
    msrs.safe_update(archmsrs)
    msrs.safe_update(machmsrs)
    return msrs

if __name__ == '__main__':
    intel_extract_msrs()