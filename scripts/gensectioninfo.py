#!/usr/bin/env python3

import os
import sys
import argparse
from string import Template

def parse_cmdline() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description='Internal build script to generate section info')
    parser.add_argument(
        '-c', '--output-cfile',
        type=str,
        required=True,
        help='Output C file'
    )
    parser.add_argument(
        '-l', '--output-linkerfile',
        type=str,
        required=True,
        help='Output linker file'
    )
    parser.add_argument(
        '-i', '--input-linkerfile',
        type=str,
        required=True,
        help='Input templated linker file'
    )
    parser.add_argument(
        'sections',
        type=str,
        nargs='+',
        help='Sections to generate info for'
    )
    return parser.parse_args()

def main() -> int:
    args = parse_cmdline()

    section_csyms = Template(
"""
// --- SECTIONINFO START: ${section} ---
extern const char*  zerOS_${section}_start;
extern const char*  zerOS_${section}_end;
extern const size_t zerOS_${section}_size;
// --- SECTIONINFO END: ${section} ---
"""
    )
    section_ldsyms = Template(
"""
/* --- SECTIONINFO START: ${section} --- */
zerOS_${section}_start = ADDR(.${section});
zerOS_${section}_end = ADDR(.${section}) + SIZEOF(.${section});
zerOS_${section}_size = SIZEOF(.${section});
/* --- SECTIONINFO END: ${section} --- */
"""
    )
    todump_csyms: str  = ""
    todump_ldsyms: str = ""

    for sec in args.sections:
        todump_csyms  += section_csyms .substitute(section=sec)
        todump_ldsyms += section_ldsyms.substitute(section=sec)

    todump_csyms  = f"\n// --- SECTIONINFO AUTOGENERATED BY {os.path.realpath(__file__)}, START ---\n" \
                  + "#include <stddef.h>\n"                                                            \
                  + todump_csyms                                                                       \
                  + f"\n// --- SECTIONINFO AUTOGENERATED BY {os.path.realpath(__file__)}, END ---\n"
    todump_ldsyms = f"\n/* --- SECTIONINFO AUTOGENERATED BY {os.path.realpath(__file__)}, START --- */\n" \
                  + todump_ldsyms                                                                         \
                  + f"\n/* --- SECTIONINFO AUTOGENERATED BY {os.path.realpath(__file__)}, END --- */\n"
    
    with open(args.output_cfile, 'w') as f:
        f.write(todump_csyms)
        f.flush()
        f.close()

    with open(args.input_linkerfile, 'r') as fr:
        with open(args.output_linkerfile, 'w') as fw:
            fw.write(fr.read().replace("!!!__GENSECTION_FILL__!!!", todump_ldsyms))
            fw.flush()
            fw.close()
        fr.close()

    return 0

def main_wrapper() -> None:
    try:
        sys.exit(main())
    except Exception as e:
        import traceback
        traceback.print_exc()
        sys.exit(1)

if __name__ == '__main__':
    main_wrapper()

    