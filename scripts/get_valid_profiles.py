#!/usr/bin/env python3

import tomllib

def valid_profiles(filepath: str) -> list[str]:
    with open(filepath, 'rb') as cargo_toml:
        return list(tomllib.load(cargo_toml).get('profile', {}).keys())

if __name__ == '__main__':
    import sys
    valid = valid_profiles(sys.argv[1])
    print(':'.join(valid), end='')
    exit(0)