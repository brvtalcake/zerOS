#!/usr/bin/env python3

from __future__ import annotations
import typing
import types
import importlib
import inspect
import mpire as mp
from mpire.async_result import AsyncResult
import errprint
import sys
import atexit
import dill
import copyreg
import pickle
import pickletools
import traceback
import weakref

from typing import Any, Self, TypeVar, Generic, Callable, TypeAlias

def _mangle_private_name(name: str, classname: str | None = None) -> str:
    if classname is not None:
        return f'_{classname.strip().strip("_")}__{name.strip().strip("_")}'
    return f'_{name.strip().strip("_")}'

MPPool: TypeAlias = mp.WorkerPool
MPAsyncResult: TypeAlias = AsyncResult

_GlobalPool: MPPool | None = None
end_worker_pool: weakref.finalize | None = None

def _end_worker_pool() -> None:
    global _GlobalPool
    if _GlobalPool is not None:
        _GlobalPool.terminate()
        _GlobalPool.join()
    return None

class Importer(object):
    def __init__(self: Self) -> None:
        self.pool = weakref.ref(_GlobalPool)

        self.processed_modules_async = []
        self.imported_modules_async = {}

        self.processed_modules = []
        self.imported_modules = {}
        return None
    
    def __repr__(self) -> str:
        return f"Importer:\n" + \
            "{\n" + \
            f"    pool: {self.pool()}\n" + \
            f"    processed_modules: {self.processed_modules}\n" + \
            f"    processed_modules_async: {self.processed_modules_async}\n" + \
            f"    imported_modules: {self.imported_modules}\n" + \
            f"    imported_modules_async: {self.imported_modules_async}\n" + \
            "}\n"
    
    def async_state(self: Self, module: str) -> tuple[MPAsyncResult | types.ModuleType, bool]:
        firstret: Any = None
        if module in self.processed_modules:
            firstret = self.imported_modules.get(module)
        elif module in self.processed_modules_async:
            firstret = self.imported_modules_async.get(module)
        if firstret is None:
            raise Exception(f'No module {module} found in importer')
        return (
            firstret,
            module in self.processed_modules_async
        )

    def do_import_classic_async(self: Self, modules: list[str]) -> typing.Tuple[ImportedModule, ...]:
        p = self.pool()
        assert p is not None
        ret = tuple()
        for module in modules:
            if module not in self.processed_modules_async and module not in self.processed_modules:
                self.imported_modules_async[module] = p.apply_async(importlib.import_module, (module,))
                self.processed_modules_async.append(module)
            ret += (ImportedModule(module, self),)
        return ret
    
    def do_import_from_async(self: Self, module: str, names: list[str]) -> typing.Tuple[ImportedObject, ...]:
        p = self.pool()
        assert p is not None
        ret = tuple()
        if module not in self.processed_modules_async and module not in self.processed_modules:
            self.imported_modules_async[module] = p.apply_async(importlib.import_module, (module,))
            self.processed_modules_async.append(module)
        for name in names:
            ret += (ImportedObject(ImportedModule(module, self), name),)
        return ret
    
    def do_import_classic(self: Self, modules: list[str]) -> typing.Tuple[ImportedModule, ...]:
        ret = tuple()
        for module in modules:
            if module not in self.processed_modules_async and module not in self.processed_modules:
                self.imported_modules[module] = importlib.import_module(module)
                self.processed_modules.append(module)
            ret += (ImportedModule(module, self),)
        return ret

class ImportedModule(object):
    def __init__(self: Self, module: str, importer_instance: Importer) -> None:
        object.__setattr__(self, _mangle_private_name('__import_parallel__module', 'ImportedModule'), module)
        object.__setattr__(self, _mangle_private_name('__import_parallel__importer_instance', 'ImportedModule'), importer_instance)
        return None

    def __get_async_state(self: Self) -> tuple[MPAsyncResult | types.ModuleType, bool]:
        return object.__getattribute__(self, _mangle_private_name('__import_parallel__importer_instance', 'ImportedModule')).async_state(object.__getattribute__(self, _mangle_private_name('__import_parallel__module', 'ImportedModule')))
        

    def __get_instance(self: Self) -> types.ModuleType:
        state, is_async = self.__get_async_state()
        if is_async:
            while not state.ready():
                continue
            real: types.ModuleType = state.get()
            if not state.successful():
                raise Exception(f'Parallel import of module {object.__getattribute__(self, _mangle_private_name("__import_parallel__module", "ImportedModule"))} failed')
            return real
        else:
            assert isinstance(state, types.ModuleType)
            real: types.ModuleType = state
            return real
    
    def __getattr__(self, name):
        return getattr(self.__get_instance(), name)

    def __setattr__(self, name, value):
        setattr(self.__get_instance(), name, value)
        return None
    
    def __delattr__(self, name):
        delattr(self.__get_instance(), name)
        return None
    
    def __dir__(self):
        return dir(self.__get_instance())

class ImportedObject(object):
    def __init__(self: Self, imported_module: ImportedModule, name: str) -> None:
        object.__setattr__(self, _mangle_private_name('__import_parallel__imported_module', 'ImportedObject'), imported_module)
        object.__setattr__(self, _mangle_private_name('__import_parallel__name', 'ImportedObject'), name)
        return None
    
    def __get_imported_module(self: Self) -> ImportedModule:
        return object.__getattribute__(self, _mangle_private_name('__import_parallel__imported_module', 'ImportedObject'))
    
    def __get_name(self: Self) -> str:
        return object.__getattribute__(self, _mangle_private_name('__import_parallel__name', 'ImportedObject'))
    
    def __get_instance(self: Self) -> Any:
        return getattr(self.__get_imported_module(), self.__get_name())
    
    def __getattr__(self, name):
        return getattr(self.__get_instance(), name)
    
    def __setattr__(self, name, value):
        setattr(self.__get_instance(), name, value)
        return None
    
    def __delattr__(self, name):
        delattr(self.__get_instance(), name)
        return None
    
    def __dir__(self):
        return dir(self.__get_instance())
    
_T = TypeVar('_T')

# To be used with values returned by from_module_import_async
class TypeProxy(Generic[_T]):
    def __init__(self: Self, imported: ImportedObject) -> None:
        self.m_impobj = imported
        return None
    
    def __call__(self: Self) -> _T:
        return type(self).get_obj(self.m_impobj)
    
    @staticmethod
    def get_obj(imported: ImportedObject) -> _T:
        return object.__getattribute__(imported, _mangle_private_name('__get_instance', 'ImportedObject'))()

class _Pickler(object):
    def __call__(self: Self, object: types.ModuleType) -> tuple[Callable, tuple[bytes]]:
        try:
            import dill
            return _Unpickler(), (dill.dumps(object, byref=False, recurse=True),)
        except Exception as e:
            errprint.perror(f"Couldn't pickle object `{object}`: {e}\nBacktrace:\n{traceback.format_exc()}")
            errprint.perror("Exiting...")
            sys.exit(1)
class _Unpickler(object):
    def __call__(self, pickled: bytes) -> types.ModuleType:
        try:
            import dill
            return dill.loads(pickled)
        except Exception as e:
            errprint.perror(f"Couldn't pickle object `{object}`: {e}\nBacktrace:\n{traceback.format_exc()}")
            errprint.perror("Exiting...")
            sys.exit(1)

importer: Importer | None = None
def _init_module():
    try:
        global importer
        importer = Importer()
    except:
        errprint.perror("Couldn't initialize module")
        sys.exit(1)
    return

def init_worker_pool(nprocs: int) -> None:
    global importer
    global _GlobalPool
    global end_worker_pool

    if _GlobalPool is None:
        _GlobalPool = mp.WorkerPool(n_jobs=nprocs, daemon=False, use_dill=True, enable_insights=True, start_method='spawn')
    if end_worker_pool is None and _GlobalPool is not None:
        end_worker_pool = weakref.finalize(_GlobalPool, _end_worker_pool)
    if importer is None:
        _init_module()
    return None

def import_classic(modules: list[str]) -> typing.Tuple[ImportedModule, ...]:
    global importer
    global _GlobalPool
    global end_worker_pool

    assert importer is not None
    assert _GlobalPool is not None
    assert end_worker_pool is not None

    return importer.do_import_classic(modules)

def import_async(modules: list[str]) -> typing.Tuple[ImportedModule, ...]:
    global importer
    global _GlobalPool
    global end_worker_pool

    assert importer is not None
    assert _GlobalPool is not None
    assert end_worker_pool is not None

    return importer.do_import_classic_async(modules)

def from_module_import_async(module: str, names: list[str]) -> typing.Tuple[ImportedObject, ...]:
    global importer
    global _GlobalPool
    global end_worker_pool

    assert importer is not None
    assert _GlobalPool is not None
    assert end_worker_pool is not None

    return importer.do_import_from_async(module, names)

### TESTS ###
if __name__ == '__main__':
    init_worker_pool(8)
    (re,) = import_async(['re'])
    print(importer)
    dummy = re.compile(r'.+')
    print(type(dummy))
    RegexProxy = TypeProxy[re.Pattern[str]]
    _ALNUM_REGEX = from_module_import_async('mk_escape_seq', ['ALNUM_REGEX'])[0]
    ALNUM_REGEX = RegexProxy.get_obj(_ALNUM_REGEX)
    match1 = re.match(ALNUM_REGEX, 'a')
    match2 = re.match(ALNUM_REGEX, '1')
    match3 = re.match(ALNUM_REGEX, '!')
    assert match1 is not None
    assert match2 is not None
    assert match3 is None
### END TESTS ###