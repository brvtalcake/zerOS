#!/usr/bin/env python3

import readline
import os
import sys
import subprocess
import tempfile
import io
import traceback
import argparse
import re
import enum
import typing
from typing import Union, Any, TypeAlias, Callable
from errprint import set_debug_mode, get_debug_mode, pdebug, pinfo, pwarning, perror
from mk_escape_seq import ALNUM_LIST, DIGIT_LIST, ALPHA_LIST, WHITESPACE_LIST, PUNCT_LIST, SYMBOL_LIST
import functools

def static_vars(**kwargs):
    @functools.wraps
    def decorate(func):
        for k in kwargs:
            setattr(func, k, kwargs[k])
        return func
    return decorate

class OnlyExitType(Exception):
    pass

def unimplemented(msg: str | None = None):
    perror(f"found an unimplemented code path: {msg}" if msg is not None else "found an unimplemented code path!")
    sys.exit(1)

OnlyExit = OnlyExitType()

def _handle_error() -> int:
    pwarning("Non fatal error occurred. Would you like to continue? [y/N]", end=' ')
    if input().lower() != 'y':
        perror("User aborted")
        return 1
    return 0

# TODO: Get these different library paths portably
SD_CALLABLE_CMDLINE = '#CC# -x none -O3 -ffreestanding -nostdlib -std=gnu23 /usr/lib/../lib64/crt1.o /usr/lib/../lib64/crti.o /usr/local/lib64/gcc/x86_64-solus-linux/14.1.0/crtbegin.o -L/usr/local/lib64/gcc/x86_64-solus-linux/14.1.0 -L/usr/local/lib64/gcc/x86_64-solus-linux/14.1.0/../../../../lib64 -L/lib/../lib64 -L/usr/lib/../lib64 -L/usr/local/lib64/gcc/x86_64-solus-linux/14.1.0/../../.. #IN# /usr/local/lib64/gcc/x86_64-solus-linux/14.1.0/crtend.o /usr/lib/../lib64/crtn.o -lc -lgcc -lgcc_s -Wl,-z,noexecstack -Wl,--dynamic-linker,/usr/lib64/ld-linux-x86-64.so.2 -o #OUT#'
SD_CALLABLE_CC = os.path.realpath(os.path.join(os.path.dirname(sys.argv[0]), '..', 'toolchain', 'install', 'bin', 'x86_64-elf-gcc'))

def _modify_cc_cmdline(cmdline: str) -> bool:
    global SD_CALLABLE_CMDLINE
    if '#CC#' not in cmdline:
        pwarning("No #CC# in the command line, using default")
        return False
    if '#OUT#' not in cmdline:
        pwarning("No #OUT# in the command line, using default")
        return False
    if '#IN#' not in cmdline:
        pwarning("No #IN# in the command line, using default")
        return False
    SD_CALLABLE_CMDLINE = cmdline
    return True

def _modify_cc_path(cc_path: str) -> bool:
    global SD_CALLABLE_CC
    if not os.path.exists(cc_path):
        pwarning("Specified CC path does not exist, using default")
        return False
    SD_CALLABLE_CC = cc_path
    return True

def _get_cc_cmdline(cc_path: str, infile: str, outfile: str, lang: str) -> list[str]:
    global SD_CALLABLE_CMDLINE
    reallang: str = lang.lower().strip()
    if reallang not in ['c', 'cpp', 'cxx']:
        pwarning(f"Unsupported language {lang}, using C")
        reallang = 'c'
    elif reallang in ['cxx', 'cpp']:
        reallang = 'c++'
    return SD_CALLABLE_CMDLINE.replace('#CC#', cc_path).replace('#IN#', f'-x{reallang} {infile} -x none').replace('#OUT#', outfile).split(' ')

def _get_cc_path() -> str:
    global SD_CALLABLE_CC
    return SD_CALLABLE_CC

def _get_tmpfile() -> str:
    t = tempfile.mkstemp()
    os.close(t[0])
    return t[1]


def _preprocess_backslashes(text: str) -> str:
    newtext = ''
    textlen = len(text)
    i = 0
    in_string_lit = False
    in_char_lit = False
    while i < textlen:
        match text[i]:
            case "\\":
                if in_string_lit or in_char_lit:
                    if i + 1 < textlen:
                        newtext += text[i] + text[i + 1]
                        i += 2
                    else:
                        raise ValueError("Untermianted string or character literal") # TODO: Add more info to the error
                else:
                    if i + 1 < textlen:
                        if text[i + 1] != '\n':
                            raise ValueError("Backslash outside of string or character literal must be at the end of the line")
                        i += 2
                    else:
                        i += 1
            case "\"":
                if not in_char_lit:
                    in_string_lit = not in_string_lit
                    newtext += text[i]
            case "'":
                if not in_string_lit:
                    in_char_lit = not in_char_lit
                    newtext += text[i]
            case _:
                newtext += text[i]
                i += 1
    return newtext


def _rm_clike_comments(text):
    'TODO: Test this function'
    def replacer(match):
        s = match.group(0)
        if s.startswith('/'):
            return " " # note: a space and not an empty string
        else:
            return s
    pattern = re.compile(
        r'//.*?$|/\*.*?\*/|\'(?:\\.|[^\\\'])*\'|"(?:\\.|[^\\"])*"',
        re.DOTALL | re.MULTILINE
    )
    return re.sub(pattern, replacer, text)

def _time_function(func, *args, **kwargs):
    import time
    start = time.time()
    _ret = func(*args, **kwargs)
    end = time.time()
    funcname = func.__name__
    print(f"Elapsed time during {funcname} execution: {end - start}")
    return _ret

def _timed(func):
    def _wrapper(*args, **kwargs):
        return _time_function(func, *args, **kwargs)
    return _wrapper

REGEX_FLAGS = re.UNICODE

PRAGMA_IMPORT_REGEX = re.compile(r'^\s*#\s*pragma\s+supdef\s+import\s*<(.*)>\s*$')

PRAGMA_DEFINE_START_REGEX = re.compile(r'^\s*#\s*pragma\s+supdef\s+begin\s+(\w+)\s*$')
PRAGMA_DEFINE_END_REGEX   = re.compile(r'^\s*#\s*pragma\s+supdef\s+end\s*$')

PRAGMA_RUNNABLE_LANGUAGE_OPTION_REGEX = r'C|CPP|CXX|c|cpp|cxx'
PRAGMA_RUNNABLE_OPERATION_OPTION_REGEX = r'trycompile|retcode|stderr|stdout|TRYCOMPILE|RETCODE|STDERR|STDOUT'
PRAGMA_RUNNABLE_OPTIONS_REGEX = r'(%s)(\s+(%s))*' % (
    '|'.join([PRAGMA_RUNNABLE_LANGUAGE_OPTION_REGEX, PRAGMA_RUNNABLE_OPERATION_OPTION_REGEX]),
    '|'.join([PRAGMA_RUNNABLE_LANGUAGE_OPTION_REGEX, PRAGMA_RUNNABLE_OPERATION_OPTION_REGEX])
)
PRAGMA_RUNNABLE_START_REGEX = re.compile(r'^\s*#\s*pragma\s+supdef\s+runnable\s+(%s)\s+begin\s+(\w+)\s*$' % PRAGMA_RUNNABLE_OPTIONS_REGEX)
PRAGMA_RUNNABLE_END_REGEX   = PRAGMA_DEFINE_END_REGEX

class PragmaType(enum.Enum):
    IMPORT   = 1
    DEFINE   = 2
    RUNNABLE = 3

class Pragma(object):
    m_pragma_type: PragmaType
    m_name: str
    m_options: list[str] | None
    def __init__(self, pragma_type: PragmaType, name: str, options: list[str] | None = None):
        self.m_pragma_type = pragma_type
        self.m_name = name
        self.m_options = options
        return None
    def __str__(self) -> str:
        if self.m_options is not None:
            return f"Pragma({self.m_pragma_type}, {self.m_name}, {self.m_options})"
        return f"Pragma({self.m_pragma_type}, {self.m_name})"
    def __repr__(self) -> str:
        return str(self)

class ImportPragma(Pragma):
    def __init__(self, imported: str):
        super().__init__(PragmaType.IMPORT, imported)
        return None

class DefinePragma(Pragma):
    m_content: str
    def __init__(self, name: str, define_content: str):
        super().__init__(PragmaType.DEFINE, name)
        self.m_content = define_content
        return None

class RunnablePragma(Pragma):
    m_runnable: str
    m_lang: str
    m_op: str
    def __init__(self, name: str, runnable_content: str, options: list[str]):
        super().__init__(PragmaType.RUNNABLE, name, options)
        self.m_runnable = runnable_content
        self.m_lang = 'c'
        self.m_opts = 'stdout'
        lang_found: bool = False
        op_found: bool = False
        if self.m_options is not None:
            for opt in self.m_options:
                if opt in PRAGMA_RUNNABLE_LANGUAGE_OPTION_REGEX.split('|'):
                    if lang_found:
                        perror("Multiple languages specified for runnable pragma %s" % self.m_name)
                        sys.exit(1)
                    self.m_lang = opt.lower()
                    lang_found = True
                elif opt in PRAGMA_RUNNABLE_OPERATION_OPTION_REGEX.split('|'):
                    if op_found:
                        perror("Multiple operations specified for runnable pragma %s" % self.m_name)
                        sys.exit(1)
                    self.m_op = opt.lower()
                    op_found = True
        return None
    def is_c(self) -> bool:
        return self.m_lang == 'c'
    def opt_trycompile(self) -> bool:
        return self.m_op == 'trycompile'
    def opt_retcode(self) -> bool:
        return self.m_op == 'retcode'
    def opt_stderr(self) -> bool:
        return self.m_op == 'stderr'
    def opt_stdout(self) -> bool:
        return self.m_op == 'stdout'
    
class Invocation(object):
    def __init__(self, parent_text: str, pragma_name: str, start: int, end: int) -> None:
        self.m_parent_text = parent_text
        self.m_pragma_name = pragma_name
        self.m_start = start
        self.m_end = end
        return None
    @property
    def parent_text(self) -> str:
        return self.m_parent_text
    @property
    def pragma_name(self) -> str:
        return self.m_pragma_name
    @property
    def start(self) -> int:
        return self.m_start
    @property
    def end(self) -> int:
        return self.m_end
    def invocation_text(self) -> str:
        return self.m_parent_text[self.m_start:self.m_end + 1]
    
class RawInvocation(Invocation):
    def __init__(self, parent_text: str, pragma_name: str, start: int, end: int, argstart: int, argend: int) -> None:
        super().__init__(parent_text, pragma_name, start, end)
        self.m_argstart = argstart
        self.m_argend = argend
        return None
    @property
    def argstart(self) -> int:
        return self.m_argstart
    @property
    def argend(self) -> int:
        return self.m_argend
    def argtext(self) -> str:
        return self.m_parent_text[self.m_argstart:self.m_argend + 1]

class ProcessedInvocation(Invocation):
    def __init1(self, parent_text: str, pragma_name: str, start: int, end: int, args: list[str], result: str) -> None:
        super().__init__(parent_text, pragma_name, start, end)
        self.m_args = args
        self.m_result = result
        return None
    def __init2(self, raw: Invocation, args: list[str], result: str) -> None:
        super().__init__(raw.m_parent_text, raw.m_pragma_name, raw.m_start, raw.m_end)
        self.m_args = args
        self.m_result = result
        return None
    def __init__(self, *args, **kwargs) -> None:
        if len(args) == 6:
            self.__init1(*args)
        elif len(args) == 3:
            self.__init2(*args)
        else:
            raise ValueError("Invalid number of arguments")
        return None
    @property
    def args(self) -> list[str]:
        return self.m_args
    @property
    def result(self) -> str:
        return self.m_result

class InvocationParser(object):
    def __init__(self, text: str, known_idents: list[str], get_prag: Callable[[str], Pragma | None], driver_instance) -> None:
        self.m_text = text
        self.m_known_idents = known_idents
        self.m_get_prag_func = get_prag
        self.m_driver = driver_instance
        return
    
    def _get_prag(self, name: str) -> Pragma:
        maybeprag = self.m_get_prag_func(name)
        if maybeprag is None:
            raise RuntimeError("Can't find pragma: %s" % (name))
        return maybeprag
    
    def _substitute(self, name: str, args: list[str]) -> str:
        return self.m_driver._process_replaceable_pragma(self._get_prag(name), args)
    
    def _basic_substitute_at(self, res: str, into: str, start: int, end: int) -> str:
        return into[0:start] + res + into[end + 1:len(into)]
    
    def _substitute_at(self, name: str, args: list[str], into: str, start: int, end: int) -> str:
        return self._basic_substitute_at(self._substitute(name, args), into, start, end)
    
    def _char_is_identifier(self, c: str, first: bool = False) -> bool:
        if first:
            return ord(c) in ALPHA_LIST or ord(c) == ord('_')
        return ord(c) in ALNUM_LIST or ord(c) == ord('_')
    def _str_is_known_identifier(self, ident: str) -> bool:
        return ident in self.m_known_idents
    def _str_is_identifier(self, ident: str) -> bool:
        return self._char_is_identifier(ident[0], True) and all([self._char_is_identifier(c) for c in ident[1:]])

    def _char_is_whitespace(self, c: str) -> bool:
        return ord(c) in WHITESPACE_LIST
    def _str_is_whitespace(self, s: str) -> bool:
        return all([self._char_is_whitespace(c) for c in s])
    
    def _get_next_invocation(self, text: str | None = None, start: int = 0) -> RawInvocation | None:
        text = text or self.m_text
        textlen = len(text)
        i = start
        in_string_lit = False
        in_char_lit = False
        potential_ident = None
        start = -1
        end = -1
        while i < textlen:
            if self._char_is_whitespace(text[i]):
                i += 1
                continue
            if self._char_is_identifier(text[i], potential_ident is None):
                potential_ident = (potential_ident or '') + text[i]
                i += 1
                continue
            if potential_ident is not None:
                if self._str_is_known_identifier(potential_ident):
                    start = i - len(potential_ident)
                    break
                potential_ident = None
                i += 1
                continue
            if text[i] == "\\":
                if not (in_string_lit or in_char_lit):
                    raise RuntimeError("Backslash outside of string or character literal should be processed at this stage")
                else:
                    i += 2
                continue
            if text[i] == "\"":
                if not in_char_lit:
                    in_string_lit = not in_string_lit
                i += 1
                continue
            if text[i] == "'":
                if not in_string_lit:
                    in_char_lit = not in_char_lit
                i += 1
                continue
            i += 1
        if start == -1:
            return None
        assert potential_ident is not None
        openparens = 0
        j = start
        argstart = -1
        argend   = -1
        while j < textlen:
            if text[j] == '(':
                if argstart == -1:
                    argstart = j
                openparens += 1
            elif text[j] == ')':
                openparens -= 1
                if openparens == 0:
                    end = j
                    argend = j
                    break
            j += 1
        if end == -1:
            raise ValueError("Unterminated invocation")
        return RawInvocation(text, potential_ident.strip(), start, end, argstart + 1, argend - 1)
    
    def _get_next_arg(self, text: str, start: int) -> tuple[int, int]:
        textlen = len(text)
        i = start
        in_string_lit = False
        in_char_lit = False
        openparens = 0
        while i < textlen:
            if text[i] == '(' and not (in_string_lit or in_char_lit):
                openparens += 1
            elif text[i] == ')' and not (in_string_lit or in_char_lit):
                openparens -= 1
            elif text[i] == ',' and openparens == 0 and not (in_string_lit or in_char_lit):
                return (start, i)
            elif text[i] == "\\":
                if not (in_string_lit or in_char_lit):
                    raise RuntimeError("Backslash outside of string or character literal should be processed at this stage")
                else:
                    i += 1
            elif text[i] == "\"":
                if not in_char_lit:
                    in_string_lit = not in_string_lit
            elif text[i] == "'":
                if not in_string_lit:
                    in_char_lit = not in_char_lit
            i += 1
        return (start, i)
    
    def _parse_invocation(self, raw: RawInvocation) -> ProcessedInvocation:
        args: list[str] = []
        expanded_args: list[str] = []
        result: str = ''
        nextarg_bounds: tuple[int, int] = (0, 0)
        text = raw.argtext()
        raw_argslen = len(text)
        while True:
            arg = self._get_next_arg(text, nextarg_bounds[1])
            if arg[0] >= raw_argslen or arg[1] >= raw_argslen:
                break
            argtext = text[arg[0]:arg[1]]
            args.append(argtext)
            expanded_argtext: str
            if len(argtext) < 3:
                expanded_argtext = argtext
            else:
                expanded_argtext = argtext
                maybenested = self._get_next_invocation(expanded_argtext, 0)
                while maybenested is not None:
                    processed = self._parse_invocation(maybenested)
                    expanded_argtext = self._basic_substitute_at(processed.result, expanded_argtext, processed.start, processed.end)
                    maybenested = self._get_next_invocation(expanded_argtext, 0)
            expanded_args.append(expanded_argtext)
        unimplemented()
                    
    #def _parse_args(self, args: str) -> list[str]:
    #    """
    #    Precondition(s) : `args` is a string containing all the text in the invocation left-paren and its corresponding right-paren.
    #    Postcondition(s): Returns a list of strings containing the arguments of the invocation. All arguments containing other invocations are treated and expanded.
    #    """
    #    argslen = len(args)
    #    i = 0
    #    current_arg: str | None = None
    #    in_string_lit = False
    #    in_char_lit = False
    #    openparens = 0
    #    ret: list[str] = []
    #    while i < argslen:
    #        if current_arg is None:
    #            if self._char_is_whitespace(args[i]):
    #                i += 1
    #                continue
    #            if args[i] == ',':
    #                ret.append('')
    #                i += 1
    #                continue
    #            current_arg = ''
    #        if args[i] == '(':
    #            openparens += 1
    #        elif args[i] == ')':
    #            openparens -= 1
    #        if openparens == 0:
    #            if args[i] == ',':
    #                ret.append(current_arg)
    #                current_arg = None
    #            else:
    #                current_arg += args[i]
    #        else:
    #            current_arg += args[i]
    #        i += 1
    #    if current_arg is not None:
    #        ret.append(current_arg)
    #    else:
    #        ret.append('')
    #    return ret
    #def _parse_invocation(
    #    content: str,
    #    pragma_names: list[str],
    #    i: int,
    #    invocations: list[tuple[int, tuple[int, int], str, str]],
    #    known_ident: str
    #) -> tuple[int, list[tuple[int, tuple[int, int], str, str]]]:
    #    def _search_invocations(args: list[str]) -> list[int]:
    #        '''
    #        In the currently parsed invocation, search for other invocations,
    #        and return the argument index and the resulting content. Code should
    #        be similar to the one in _parse_invocations. All nested invocations are
    #        passed recursively to this _parse_invocation function to get the invocation results.
    #        '''
    #        nargs = len(args)
    #        i = 0
    #        while i < nargs:
    #            in_string_lit = False
    #            in_char_lit = False
    #            potential_ident = None
    #            arglen = len(args[i])
    #            argn = args[i]
    #            j = 0
    #            while j < arglen:
    #                if self._char_is_whitespace(argn[j]):
    #                    j += 1
    #                    continue
    #                if self._char_is_identifier(argn[j], potential_ident is None):
    #                    potential_ident = (potential_ident or '') + argn[j]
    #                    j += 1
    #                    continue
    #                if potential_ident is not None:
    #                    if self._str_is_known_identifier(potential_ident, pragma_names):
    #                        j, invocations = _parse_invocation(argn, pragma_names, j, invocations, potential_ident)
    #                    else:
    #                        potential_ident = None
    #                        j += 1
    #                    continue
    #                if argn[j] == "\\":
    #                    if not (in_string_lit or in_char_lit):
    #                        raise RuntimeError("Backslash outside of string or character literal should be processed at this stage")
    #                    else:
    #                        j += 2
    #                    continue
    #                if argn[j] == "\"":
    #                    if not in_char_lit:
    #                        in_string_lit = not in_string_lit
    #                    j += 1
    #                    continue
    #                if argn[j] == "'":
    #                    if not in_string_lit:
    #                        in_char_lit = not in_char_lit
    #                    j += 1
    #                    continue
    #                j += 1                        
    #    invocation = (0,)
    #    # Search parenthesis
    #    start = -1
    #    end = -1
    #    openparens = 0
    #    j = i
    #    while j < len(content):
    #        if content[j] == '(':
    #            if start == -1:
    #                start = j
    #            openparens += 1
    #        elif content[j] == ')':
    #            openparens -= 1
    #            if openparens == 0:
    #                end = j
    #                break
    #        j += 1
    #    if start == -1 or end == -1:
    #        raise ValueError("Unterminated invocation")
    #    args = _parse_args(content[start + 1:end].strip())
    #def _parse_invocations(content: str, pragma_names: list[str]) -> list[tuple[int, tuple[int, int], str, str]]:
    #    '''
    #    We must handle nested calls such as:
    #    MACRO1(arg1, MACRO2(arg2, arg3), "arg4", ...)
#
    #    We must also handle things like:
    #    MACRO1     (arg1, arg2,
    #        arg3    , arg4)
#
    #    And of course, we must also handle classic invocations like:
    #    MACRO1(arg1, arg2, arg3, arg4)
    #    '''
    #    i: int = 0
    #    in_string_lit: bool = False
    #    in_char_lit: bool = False
    #    potential_ident: str | None = None
    #    invocations: list[tuple[int, tuple[int, int], str, str]] = []
    #    while i < len(content):
    #        if self._char_is_whitespace(content[i]):
    #            i += 1
    #            continue
    #        if self._char_is_identifier(content[i], potential_ident is None):
    #            potential_ident = (potential_ident or '') + content[i]
    #            i += 1
    #            continue
    #        if potential_ident is not None:
    #            if self._str_is_known_identifier(potential_ident, pragma_names):
    #                i, invocations = _parse_invocation(content, pragma_names, i, invocations, potential_ident)
    #            else:
    #                potential_ident = None
    #                i += 1
    #            continue
    #        if content[i] == "\\":
    #            if not (in_string_lit or in_char_lit):
    #                raise RuntimeError("Backslash outside of string or character literal should be processed at this stage")
    #            else:
    #                i += 2
    #            continue
    #        if content[i] == "\"":
    #            if not in_char_lit:
    #                in_string_lit = not in_string_lit
    #            i += 1
    #            continue
    #        if content[i] == "'":
    #            if not in_string_lit:
    #                in_char_lit = not in_char_lit
    #            i += 1
    #            continue
    #        i += 1
    #    return invocations

class FileContent(object):
    DefineApplier: TypeAlias = Callable[..., str]

    def __init__(self, filepath: str):
        self.m_filepath = filepath
        self.m_content = []
        self.m_imports = []
        return None
    
    def get_file_content(self, file_path: str) -> None:
        with open(file_path, 'r') as file:
            noeolbackslashes: str = _preprocess_backslashes(file.read())
            noclikecomments: str = _rm_clike_comments(noeolbackslashes)
            lines: list[str] = noclikecomments.split('\n')
            i: int = 0
            line: str
            while i < len(lines):
                line = lines[i]
                matchimport = re.match(PRAGMA_IMPORT_REGEX, line)
                matchdefine = re.match(PRAGMA_DEFINE_START_REGEX, line)
                matchrunnable = re.match(PRAGMA_RUNNABLE_START_REGEX, line)
                if matchimport:
                    pdebug(f"Found import pragma at line {i + 1}")
                    self.m_content.append((i + 1, ImportPragma(matchimport.group(1).strip())))
                    pdebug(f"Imported file: {matchimport.group(1).strip()}")
                    i += 1
                elif matchdefine:
                    pdebug(f"Found define pragma at line {i + 1}")
                    define_content: str = ''
                    j: int = i + 1
                    while j < len(lines):
                        if re.match(PRAGMA_DEFINE_END_REGEX, lines[j]):
                            break
                        define_content += lines[j]
                        j += 1
                    self.m_content.append((i + 1, DefinePragma(matchdefine.group(1).strip(), define_content)))
                    pdebug(f"Define name: {matchdefine.group(1).strip()}")
                    pdebug(f"Define content: {define_content}")
                    i = j + 1
                elif matchrunnable:
                    pdebug(f"Found runnable pragma at line {i + 1}")
                    runnable_content: str = ''
                    j: int = i + 1
                    while j < len(lines):
                        if re.match(PRAGMA_RUNNABLE_END_REGEX, lines[j]):
                            break
                        runnable_content += lines[j]
                        j += 1
                    popts = [x.strip() for x in matchrunnable.group(1).split(' ')]
                    pname = matchrunnable.groups()[-1].strip()
                    self.m_content.append((i + 1, RunnablePragma(pname, runnable_content, popts)))
                    pdebug(f"Runnable name: {pname}")
                    pdebug(f"Runnable content: {runnable_content}")
                    pdebug(f"Options: {popts}")
                    i = j + 1
                else:
                    self.m_content.append((i + 1, line))
                    i += 1
        return None
    def _process_import_pragma(self, pragma: ImportPragma):
        return FileContent(pragma.m_name)
    def _process_replaceable_pragma(self, pragma: Pragma, args: list[str]) -> str:
        if isinstance(pragma, DefinePragma) and pragma.m_pragma_type == PragmaType.DEFINE:
            return self._process_define_pragma(pragma, args)
        if isinstance(pragma, RunnablePragma) and pragma.m_pragma_type == PragmaType.RUNNABLE:
            return self._process_runnable_pragma(pragma, args)
        raise RuntimeError("Unknown pragma kind")
    def _process_define_pragma(self, pragma: DefinePragma, args: list[str]) -> str:
        '''TODO: Move this code into DefinePragma'''
        pcontent: str = pragma.m_content
        for i, arg in enumerate(args, 1):
            pcontent = pcontent.replace(f"${i}", arg)
        return pcontent
    def _process_runnable_pragma(self, pragma: RunnablePragma, args: list[str]) -> str:
        '''TODO: Move this code into RunnablePragma'''
        if not pragma.is_c():
            pwarning("For now, only supported language is C")
            perror("Unsupported language. Aborting")
            sys.exit(1)
        pcontent: str = pragma.m_runnable
        for i, arg in enumerate(args, 1):
            pcontent = pcontent.replace(f"${i}", arg)
        tmpfilein = _get_tmpfile()
        with open(tmpfilein, 'w') as file:
            file.write(pcontent)
            file.flush()
            file.close()
        tmpfileout = _get_tmpfile()
        cc_cmdline = _get_cc_cmdline(_get_cc_path(), tmpfilein, tmpfileout, pragma.m_lang)
        pinfo(f"Running command: {' '.join(cc_cmdline)}")
        pinfo(f"Input file: {tmpfilein}")
        pinfo(f"Output file: {tmpfileout}")
        pdebug(f"Content: {pcontent}")
        pdebug(f"Args: {args}")
        pdebug(f"Options: {pragma.m_options}")
        try:
            subprocess.run(cc_cmdline, check=True)
        except subprocess.CalledProcessError as e:
            perror(f"An error occurred: {e}")
            sys.exit(1)
        if pragma.opt_trycompile():
            return '1'
        pdebug(f"Compiling {pragma.m_name} succeeded")
        exe_ret = subprocess.run(tmpfileout, check=False, capture_output=True)
        pdebug(f"Return code of pragma {pragma.m_name}: {exe_ret.returncode}")
        pdebug(f"Stdout of pragma {pragma.m_name}: {exe_ret.stdout.decode('utf-8')}")
        pdebug(f"Stderr of pragma {pragma.m_name}: {exe_ret.stderr.decode('utf-8')}")
        if pragma.opt_stdout():
            return exe_ret.stdout.decode('utf-8')
        if pragma.opt_retcode():
            return f'{exe_ret.returncode}'
        if pragma.opt_stderr():
            return exe_ret.stderr.decode('utf-8')
        return exe_ret.stdout.decode('utf-8')
    def process_imports(self) -> None:
        for line in self.m_content:
            if isinstance(line[1], Pragma):
                pragma: Any = line[1]
                if pragma.m_pragma_type == PragmaType.IMPORT:
                    self.m_imports.append(self._process_import_pragma(pragma)) # TODO: Add some code to handle inclusion loops
        for imp in self.m_imports:
            imp.get_file_content()
            imp.process_imports()
    def output_processed_content(self, output_file: str | None = None) -> None:
        realoutput: typing.TextIO
        if output_file is None:
            if not isinstance(sys.stdout, typing.TextIO):
                perror("No output file specified and stdout is not available")
                sys.exit(1)
            else:
                realoutput = sys.stdout
        else:
            realoutput = open(output_file, 'w')
        #replaceable_pragma_names: list[str] = [p.m_name for (l, p) in self.m_content if (isinstance(p, DefinePragma) or isinstance(p, RunnablePragma))]
        def _get_all_supdefs(startinstance) -> list[str]:
            def _append_if_not_exists(xs, value):
                if value not in xs:
                    xs.append(value)
                return xs
            ret: list[str] = []
            l = [p.m_name for (_, p) in startinstance.m_content if (isinstance(p, DefinePragma) or isinstance(p, RunnablePragma))]
            for it in l:
                ret = _append_if_not_exists(ret, it)
            for imp in startinstance.m_imports:
                ret += _get_all_supdefs(imp)
            return ret
        replaceable_pragma_names: list[str] = _get_all_supdefs(self)
        def _find_pragma_by_name(name: str, instance = self) -> Pragma | None:
            for (_, p) in instance.m_content:
                if isinstance(p, Pragma) and p.m_name == name:
                    return p
            ret = None
            for imp in instance.m_imports:
                ret = _find_pragma_by_name(name, imp)
                if ret is not None:
                    return ret
            return None
        
        unified_content: str = '\n'.join([p if isinstance(p, str) else '' for (_, p) in self.m_content])
        #FUNCCALL_REGEX = re.compile(r'^(.*\s)?(%s)\s*\((.*)\)\s+(.*)' % '|'.join(replaceable_pragma_names))
        pdebug(f"Unified content: {unified_content}")
        pdebug(f"Replaceable pragma names: {'|'.join(replaceable_pragma_names)}")

        parser = InvocationParser(unified_content, replaceable_pragma_names, lambda s: _find_pragma_by_name(s, self), self)
        next = parser._get_next_invocation()
        if next is not None:
            pinfo(f"invocation text: {next.invocation_text()}")
            pinfo(f"arg text: {next.argtext()}")
            pinfo(f"pragma name: {next.pragma_name}")
        else:
            perror("INVOCATION NOT FOUND!")
        
        unimplemented()
        # list[(line, (start, end), pragma_name, result)]
        #macro_calls: list[tuple[int, tuple[int, int], str, str]] = _parse_invocations(unified_content, replaceable_pragma_names)

        #while True:
        #    matched = re.search(FUNCCALL_REGEX, unified_content)
        #    if matched is None:
        #        break
            #for group in matched.groups():
            #    print(f"Group: {group}")
            #continue
            #pragmaname = matched.group(1)
            #args = matched.group(2).split(',')
            #processed_content = ''
            #pragma = _find_pragma_by_name(pragmaname)
            #if pragma is None:
            #    perror(f"Pragma {pragmaname} not found")
            #    sys.exit(1)
            #if isinstance(pragma, DefinePragma):
            #    processed_content = self._process_define_pragma(pragma, args)
            #elif isinstance(pragma, RunnablePragma):
            #    processed_content = self._process_runnable_pragma(pragma, args)
            #else:
            #    perror(f"Unsupported pragma type {pragma.m_pragma_type}")
            #    sys.exit(1)
            #unified_content = unified_content[:matched.start()] + f' {processed_content} ' + unified_content[matched.end():]
        realoutput.write(unified_content)
        realoutput.close()
        return None


def process_file(file_path: str, output_file: str | None = None) -> None:
    instance: FileContent = FileContent(file_path)
    instance.get_file_content(file_path)
    instance.process_imports()
    instance.output_processed_content(output_file)
    return None

def parse_cmdline():
    parser = argparse.ArgumentParser(
        prog='supdef.py',
        description='SupDef preprocessor',
        epilog='This script is a kind of super-preprocessor for C/C++ files'
    )
    parser.add_argument(
        '-v', '--version',
        action='version',
        version='%(prog)s 0.1'
    )
    parser.add_argument(
        '-I', '--include',
        metavar='include-path',
        type=str,
        help='specify include paths',
        required=False,
        action='append'
    )
    parser.add_argument(
        '-d', '--debug',
        action='store_true',
        help='enable debug mode',
        required=False
    )
    parser.add_argument(
        '-o', '--output', '--output-file',
        metavar='output',
        type=str,
        help='output file',
        required=False
    )
    parser.add_argument(
        '--cc',
        metavar='cc-path',
        type=str,
        help='path to the C compiler',
        required=False
    )
    parser.add_argument(
        '--cc-cmdline',
        metavar='cc-cmdline',
        type=str,
        help='command line for the C compiler',
        required=False
    )
    parser.add_argument(
        'input',
        metavar='input',
        type=str,
        help='input file'
    )
    return parser.parse_args()

@_timed
def main() -> int:
    args = parse_cmdline()
    if args.debug:
        set_debug_mode(True)
    
    pdebug("Debug mode enabled")
    pdebug(f"Args: {args}")
    pdebug(f"CC: {_get_cc_path()}")
    pdebug(f"CC cmdline: {_get_cc_cmdline(_get_cc_path(), 'input', 'output', 'c')}")
    pdebug(f"PRAGMA_IMPORT_REGEX: {PRAGMA_IMPORT_REGEX}")
    pdebug(f"PRAGMA_DEFINE_START_REGEX: {PRAGMA_DEFINE_START_REGEX}")
    pdebug(f"PRAGMA_DEFINE_END_REGEX: {PRAGMA_DEFINE_END_REGEX}")
    pdebug(f"PRAGMA_RUNNABLE_START_REGEX: {PRAGMA_RUNNABLE_START_REGEX}")
    pdebug(f"PRAGMA_RUNNABLE_END_REGEX: {PRAGMA_RUNNABLE_END_REGEX}")
    pdebug(f"PRAGMA_RUNNABLE_OPTIONS_REGEX: {PRAGMA_RUNNABLE_OPTIONS_REGEX}")
    pdebug(f"PRAGMA_RUNNABLE_LANGUAGE_OPTION_REGEX: {PRAGMA_RUNNABLE_LANGUAGE_OPTION_REGEX}")
    pdebug(f"PRAGMA_RUNNABLE_OPERATION_OPTION_REGEX: {PRAGMA_RUNNABLE_OPERATION_OPTION_REGEX}")

    if args.cc_cmdline:
        if not _modify_cc_cmdline(args.cc_cmdline):
            if _handle_error() != 0:
                return 1
    if args.cc:
        if not _modify_cc_path(args.cc):
            if _handle_error() != 0:
                return 1
    try:
        process_file(args.input, args.output)
    except OnlyExitType:
        return 1
    except Exception as e:
        perror(f"An error occurred: {e}")
        pdebug(traceback.format_exc())
        return 1
    return 0

if __name__ == '__main__':
    sys.exit(main())
