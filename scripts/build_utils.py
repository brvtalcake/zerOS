import os

def do_is_modified_after(source_file_path: str, target_file_path: str) -> bool:
    r'''
    Returns `True` if the last modification time of `source_file_path` is higher than the one of `target_file_path`, else `False`.
    Provided paths must be absolute.
    '''
    return os.stat(source_file_path, follow_symlinks=False).st_mtime_ns >= os.stat(target_file_path, follow_symlinks=False).st_mtime_ns

def is_modified_after(source_file_path: str, target_file_path: str) -> bool:
    r'''
    Returns `True` if the last modification time of `source_file_path` is higher than the one of `target_file_path`, else `False`.
    Provided paths don't need to be absolute.
    '''
    return do_is_modified_after(os.path.realpath(source_file_path), os.path.realpath(target_file_path))

def needs_regen(target_file_path: str, deps_file_paths: list[str] = []) -> bool:
    real_target: str = os.path.realpath(target_file_path)
    if len(deps_file_paths) <= 0 or not os.path.exists(real_target):
        return True
    for dep in deps_file_paths:
        real_dep: str = os.path.realpath(dep)
        if do_is_modified_after(real_dep, real_target):
            return True
    return False
    