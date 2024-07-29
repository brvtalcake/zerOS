#!/usr/bin/env python3

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
from typing import Union, Any
from errprint import set_debug_mode, get_debug_mode, pdebug, pinfo, pwarning, perror

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

class FileContent(object):
    m_filepath: str
    m_content: list[tuple[int, Union[str, Pragma]]]
    m_imports: list[Any]
    def __init__(self, filepath: str):
        self.m_filepath = filepath
        self.m_content = []
        self.m_imports = []
        return None
    def get_file_content(self, file_path: str) -> None:
        # TODO: Don't forget to process all `\` characters as well
        with open(file_path, 'r') as file:
            lines: list[str] = file.readlines()
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
    def _process_define_pragma(self, pragma: DefinePragma, args: list[str]) -> str:
        pcontent: str = pragma.m_content
        for i, arg in enumerate(args, 1):
            pcontent = pcontent.replace(f"${i}", arg)
        return pcontent
    def _process_runnable_pragma(self, pragma: RunnablePragma, args: list[str]) -> str:
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
        def _parse_macro_calls(content: str, pragma_names: list[str]) -> list[tuple[int, tuple[int, int], str, str]]:
            '''We must handle nested calls such as:
            MACRO1(arg1, MACRO2(arg2, arg3), arg4)
            '''
            ret: list[tuple[int, tuple[int, int], str, str]] = []
            i: int = 0
            def _find_prev_encountered_name_start(c: str, start: int) -> int:
                for k in range(start - 1, -1, -1):
                    if c[k].isspace() or c[k] in ['(', ')']:
                        return k + 1
                return 0
            while i < len(content):
                encountered_names: list[str] = []
                openparen_count: int = 0
                if content[i] == '(':
                    potential_name_start: int = _find_prev_encountered_name_start(content, i)
                    if i - potential_name_start > 0:
                        encountered_names.append(content[potential_name_start:i])
                    else:
                        continue
                    
                    openparen_count += 1
                    j: int = i + 1
                    while j < len(content):
                        if content[j] == '(':
                            openparen_count += 1
                        elif content[j] == ')':
                            openparen_count -= 1
                            if openparen_count == 0:
                                break
                        j += 1
                    if j == len(content):
                        perror("Unmatched parenthesis")
                        sys.exit(1)


        unified_content: str = '\n'.join([p if isinstance(p, str) else '' for (_, p) in self.m_content])
        #FUNCCALL_REGEX = re.compile(r'^(.*\s)?(%s)\s*\((.*)\)\s+(.*)' % '|'.join(replaceable_pragma_names))
        pdebug(f"Unified content: {unified_content}")
        pdebug(f"Replaceable pragma names: {'|'.join(replaceable_pragma_names)}")

        # list[(line, (start, end), pragma_name, result)]
        macro_calls: list[tuple[int, tuple[int, int], str, str]] = _parse_macro_calls(unified_content, replaceable_pragma_names)

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
    except Exception as e:
        perror(f"An error occurred: {e}")
        pdebug(traceback.format_exc())
        return 1
    return 0

if __name__ == '__main__':
    sys.exit(main())
