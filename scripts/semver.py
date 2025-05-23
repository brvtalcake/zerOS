import semantic_version

def comp(v1: str, v2: str) -> int:
    """Compare two semantic versions."""
    def cmp(a: semantic_version.Version, b: semantic_version.Version) -> int:
        #return (a > b) - (a < b)
        if a < b:
            return -1
        elif a == b:
            return 0
        else:
            assert a > b
            return 1
    _v1 = semantic_version.Version.coerce(v1)
    _v2 = semantic_version.Version.coerce(v2)
    return cmp(_v1, _v2)

def sort(versions: list[str], reverse: bool = False) -> list[str]:
    """Sort a list of semantic versions."""
    import semantic_version
    return sorted(versions, key=semantic_version.Version.coerce, reverse=reverse)