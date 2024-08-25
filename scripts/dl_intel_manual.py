#!/usr/bin/env python3

import sys
import requests
import bs4
import download
from bs4 import BeautifulSoup

def do_dl() -> bytes:
    linktxt: str = "Intel® 64 and IA-32 Architectures Software Developer’s Manual Combined Volumes: 1, 2A, 2B, 2C, 2D, 3A, 3B, 3C, 3D, and 4"
    pageurl: str = "https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html"
    response = requests.get(pageurl)
    bs: bs4.PageElement = BeautifulSoup(response.text, 'html.parser')
    goodlinks: list[bs4.PageElement] = bs.find_all('a', href=True, string=linktxt)
    assert len(goodlinks) == 1
    link: bs4.Tag = goodlinks[0]
    pdf = download.from_http(link['href'], True)
    return pdf

def main() -> int:
    with open(sys.argv[1], 'wb') as f:
        f.write(do_dl())
    return 0

if __name__ == "__main__":
    sys.exit(main())