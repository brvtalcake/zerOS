#!/usr/bin/env python3

import lark

def read_supdef_file(path: str) -> str:
    ret = ""
    with open(path, "r") as file:
        content = file.read()
        remaining = len(content)
        while remaining > 0:
            match content[0]:
                case "/":
                    match content[1]:
                        case "/":
                            eol = content.find("\n")
                case "'":
                    end = remaining
                    for i in range(1, remaining):
                        curr = content[i]
                        if curr == "'":
                            end = i + 1
                            break
                        elif curr == "\\":
                            i += 1
                    removed = content[:end]
                    content = content[end:]
                    ret += removed
                case "\"":
                    end = remaining
                    for i in range(1, remaining):
                        curr = content[i]
                        if curr == "\"":
                            end = i + 1
                            break
                        elif curr == "\\":
                            i += 1
                    removed = content[:end]
                    content = content[end:]
                    ret += removed
    return ret