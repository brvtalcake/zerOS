#!/usr/bin/env -S pipenv run python3

import sys
import os
import argparse
import traceback
import pathlib
import subprocess
from scripts.errprint import *

class ProcessFailedError(Exception):
    def __init__(self, process: subprocess.CompletedProcess[str]) -> None:
        super().__init__()
        self.proc = process
        return None
    

def is_newer_than(fromfile: str, tofile: str) -> bool:
    if os.path.exists(tofile):
        return (
            os.stat(fromfile, follow_symlinks=False).st_mtime >
            os.stat(tofile  , follow_symlinks=False).st_mtime
        )
    else:
        return True

def walk_files_in(dir: str):
    for path in os.scandir(dir):
        if path.is_file(follow_symlinks=False):
            yield path.path

def cmd(cmd: list[str]):
    pinfo(f'running \'{' '.join(cmd)}\'')
    return subprocess.run(cmd, encoding='utf-8')

def cmd_if_newer(cmd: list[str], **kwargs):
    if 'fromfile' in kwargs.keys() and 'tofile' in kwargs.keys():
        fromfile = kwargs['fromfile']
        tofile   = kwargs[  'tofile']
        assert isinstance(fromfile, str) and isinstance(tofile, str)
        do_it = is_newer_than(fromfile, tofile)
        if not do_it:
            pinfo(f'{fromfile} is older than {tofile}')
            pinfo(f'no need to perform command \'{' '.join(cmd)}\'')
            return None
    elif 'pairs' in kwargs.keys():
        pairs = kwargs['pairs']
        assert isinstance(pairs, list) and all(
            map(
                lambda o: (isinstance(o, tuple) and len(o) == 2) and all(
                    map(lambda item: isinstance(item, str), o)
                ), pairs
            )
        )
        do_it = any(map(lambda tp: is_newer_than(tp[0], tp[1]), pairs))
        if not do_it:
            fromfiles = ', '.join(map(lambda tp: tp[0], pairs))
            tofiles   = ', '.join(map(lambda tp: tp[1], pairs))
            pinfo(f'{fromfiles} are all respectively older than {tofiles}')
            pinfo(f'no need to perform command \'{' '.join(cmd)}\'')
    else:
        raise RuntimeError('Internal error !')
    pinfo(f'running \'{' '.join(cmd)}\'')
    return subprocess.run(cmd, encoding='utf-8')
    
def _cp_common(fromfile: str, tofile: str):
    if tofile[-1] == '/' or os.path.isdir(tofile):
        if tofile[-1] != '/':
            tofile += '/'
        tofile += os.path.basename(fromfile)
    os.makedirs(os.path.dirname(tofile), exist_ok=True)
    need_update = is_newer_than(fromfile, tofile)
    return fromfile, tofile, need_update

def cp_if_newer(fromfile: str, tofile: str):
    fromfile, tofile, need_upd = _cp_common(fromfile, tofile)
    if not need_upd:
        pinfo(f'{fromfile} is older than {tofile}')
        pinfo('no need to perform any copy')
        return tofile
    pinfo(f'copying {fromfile} to {tofile}')
    fromfd = -1
    tofd = -1
    try:
        fromfd = os.open(fromfile, os.O_RDONLY)
        tofd = os.open(tofile, os.O_WRONLY | os.O_TRUNC | os.O_CREAT)
        os.sendfile(tofd, fromfd, None, os.fstat(fromfd).st_size)
    except:
        raise
    finally:
        if fromfd != -1:
            os.close(fromfd)
        if tofd != -1:
            os.close(tofd)
    return tofile

def cp(fromfile: str, tofile: str):
    fromfile, tofile, _ = _cp_common(fromfile, tofile)
    pinfo(f'copying {fromfile} to {tofile}')
    fromfd = -1
    tofd = -1
    try:
        fromfd = os.open(fromfile, os.O_RDONLY)
        tofd = os.open(tofile, os.O_WRONLY | os.O_TRUNC | os.O_CREAT)
        os.sendfile(tofd, fromfd, None, os.fstat(fromfd).st_size)
    except:
        raise
    finally:
        if fromfd != -1:
            os.close(fromfd)
        if tofd != -1:
            os.close(tofd)
    return tofile

def cargo(cargo_cmd: str, *extra: str, **kwargs):
    args = [ 'cargo', cargo_cmd, '-Z', 'unstable-options' ]
    args.extend(extra)
    if 'features' in kwargs.keys() and len(kwargs['features']) > 0:
        args.extend(
            [
                '--no-default-features', '--features',
                f'"{' '.join(kwargs['features'])}"'
            ]
        )
    if 'profile' in kwargs.keys():
        args.extend([ '--profile', kwargs['profile'] ])
    return cmd(args)

def checkps(ps: subprocess.CompletedProcess[str]):
    if ps.returncode != 0:
        raise ProcessFailedError(ps)
    return None

def _zerOS_prebuild(features: list[str], profile: str):
    return None

def _zerOS_build(features: list[str], profile: str):
    saved = os.getcwd()
    try:
        os.chdir('./zerOS')
        ps = cargo('bamd64', '--artifact-dir', './build', features=features, profile=profile)
        checkps(ps)
        cp('./build/zerOS', './bin/')
    except:
        raise
    finally:
        os.chdir(saved)
    return None

def _zerOS_postbuild(features: list[str], profile: str):
    saved = os.getcwd()
    try:
        os.chdir('./zerOS')
        pairs = [('./bin/zerOS', './bin/zerOS.iso')]
        pairs.append(('./bin/zerOS', cp_if_newer('./bin/zerOS', './iso_root/boot/')))
        pairs.append(('./config/limine.conf', cp_if_newer('./config/limine.conf', './iso_root/boot/limine/')))
        pairs.append(('/usr/share/limine/BOOTX64.EFI', cp_if_newer('/usr/share/limine/BOOTX64.EFI', './iso_root/EFI/BOOT/')))
        for path in walk_files_in('/usr/share/limine'):
            if path.endswith('.sys') or path.endswith('.bin'):
                pairs.append((path, cp_if_newer(path, './iso_root/boot/limine/')))
        ps = cmd_if_newer(
            cmd=[
                'xorriso', '-as', 'mkisofs', '-b', 'boot/limine/limine-bios-cd.bin',
			    '-no-emul-boot', '-boot-load-size', '4', '-boot-info-table',
			    '--efi-boot', 'boot/limine/limine-uefi-cd.bin', '-efi-boot-part',
                '--efi-boot-image', '--protective-msdos-label', 'iso_root',
                '-o', './bin/zerOS.iso'
            ],
            pairs=pairs
        )
        checkps(ps) if ps is not None else None
    except:
        raise
    finally:
        os.chdir(saved)
    return None

def _macro_utils_prebuild(features: list[str], profile: str):
    return None
def _macro_utils_build(features: list[str], profile: str):
    if len(features) == 0:
        _cmd = [ 'cargo', 'build' ]
    else:
        _cmd = [ 'cargo', 'build', '--no-default-features', '--features', f'"{' '.join(features)}"']
    saved = os.getcwd()
    try:
        os.chdir('./zerOS')
        ps = cmd(_cmd)
        checkps(ps)
    except:
        raise
    finally:
        os.chdir(saved)
    return None
def _macro_utils_postbuild(features: list[str], profile: str):
    return None

def _proc_macro_utils_prebuild(features: list[str], profile: str):
    return None
def _proc_macro_utils_build(features: list[str], profile: str):
    if len(features) == 0:
        _cmd = [ 'cargo', 'build' ]
    else:
        _cmd = [ 'cargo', 'build', '--no-default-features', '--features', f'"{' '.join(features)}"']
    saved = os.getcwd()
    try:
        os.chdir('./zerOS')
        cmd(_cmd)
    except:
        raise
    finally:
        os.chdir(saved)
    return None
def _proc_macro_utils_postbuild(features: list[str], profile: str):
    return None

PACKAGE_LIST = {
    pathlib.Path('zerOS'): {
        'prebuild'  : _zerOS_prebuild,
        'build': _zerOS_build,
        'postbuild' : _zerOS_postbuild,
    },
    pathlib.Path('macro-utils'): {
        'prebuild'  : _macro_utils_prebuild,
        'build': _macro_utils_build,
        'postbuild' : _macro_utils_postbuild,
    },
    pathlib.Path('proc-macro-utils'): {
        'prebuild'  : _proc_macro_utils_prebuild,
        'build': _proc_macro_utils_build,
        'postbuild' : _proc_macro_utils_postbuild,
    },
}

def package_build(pkg: str, features: list[str], profile: str):
    path = pathlib.Path(pkg)
    if not path in PACKAGE_LIST.keys():
        raise ValueError(f'Unknown package {pkg} !')

    fn = PACKAGE_LIST[path]['prebuild']
    if fn is not None:
        fn(features, profile)
    
    fn = PACKAGE_LIST[path]['build']
    if fn is not None:
        fn(features, profile)
    
    fn = PACKAGE_LIST[path]['postbuild']
    if fn is not None:
        fn(features, profile)
    
    return None

def make_profile_string(profile: str, want_lto: bool) -> str:
    if want_lto:
        return f'{profile.strip().strip('-')}-lto'
    return profile.strip().strip('-')

def parse_cmdline():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        'project-name',
        type=str,
        nargs='+',
        help='Projects to build'
    )
    parser.add_argument(
        '-f', '--feature',
        action='append',
        help='Features to use. Optionally per-package with \'<pkg>:<feature>\' syntax'
    )
    parser.add_argument(
        '-o', '--order',
        type=str,
        default=None,
        help='Order of package compilation'
    )
    parser.add_argument(
        '-p' '--profile',
        type=str,
        default='dev',
        help='The base profile to use: release, dev, test, bench, etc...'
    )
    parser.add_argument(
        '--lto',
        action='store_true',
        default=False,
        help='Whether or not we want to compile using lto'
    )
    return parser.parse_args()

def make_feature_dicts(pkgs: list[str], feats: list[str] | None) -> dict[str, list[str]]:
    ret: dict[str, list[str]] = { }
    def do_foreach_pkg(fn):
        for pkg in pkgs:
            ret[pkg] = fn(ret.get(pkg, []))
        return None
    do_foreach_pkg(lambda _: [])
    if feats is not None:
        for feat in feats:
            if ':' not in feat:
                do_foreach_pkg(lambda old: [*old, feat])
                continue
            [pkg, actual_feat] = feat.split(':', 2)
            ret[pkg].append(actual_feat)
    return ret

def main() -> int:
    try:
        parsed = parse_cmdline()
        PROJECTS = getattr(parsed, 'project-name')
        FEATURES = getattr(parsed, 'feature') or []
        ORDER    = getattr(parsed, 'order') or PROJECTS
        PROFILE  = getattr(parsed, 'p__profile')
        LTO      = getattr(parsed, 'lto')
        PROJECTS_len = len(PROJECTS)
        ORDER_len    = None
        
        # Round 1 checks
        if not isinstance(PROJECTS, list) or PROJECTS_len == 0:
            raise ValueError('no value provided for project names')
        if isinstance(ORDER, str):
            ORDER = ORDER.split(',')
            ORDER_len = len(ORDER)
        elif isinstance(ORDER, list):
            ORDER_len = len(ORDER)
        else:
            raise RuntimeError('Internal error !')
        
        if ORDER_len != PROJECTS_len:
            raise ValueError('Mismatch of specified projects count in order list')
        
        # Round 2 checks
        for i, obj in enumerate(PROJECTS):
            for j in range(i + 1, PROJECTS_len):
                if PROJECTS[j] == obj:
                    raise ValueError(f'Project name {obj} specified more than one time !')
        for i, obj in enumerate(ORDER):
            for j in range(i + 1, ORDER_len):
                if ORDER[j] == obj:
                    raise ValueError(f'{obj} specified more than one time (in order-list) !')
        
        feat_dict = make_feature_dicts(PROJECTS, FEATURES)
        profile = make_profile_string(PROFILE, LTO)
        for pkg in ORDER:
            package_build(pkg, feat_dict[pkg], profile)
    except ProcessFailedError as e:
        perror(f'command \'{' '.join(e.proc.args)}\' exited with error code {e.proc.returncode}')
        perror("terminating...")
    except Exception:
        perror(traceback.format_exc())
        perror("terminating...")
        return 1
    return 0

if __name__ == '__main__':
    sys.exit(main())
