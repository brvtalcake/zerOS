PDEBUG_START:   str = "\033[1;34m  [DEBUG]\033[0m  "
PDEBUG_END:     str = "\n"
PINFO_START:    str = "\033[1;32m  [INFO ]\033[0m  "
PINFO_END:      str = "\n"
PWARNING_START: str = "\033[1;33m  [WARN ]\033[0m  "
PWARNING_END:   str = "\n"
PERROR_START:   str = "\033[1;31m  [ERROR]\033[0m  "
PERROR_END:     str = "\n"

LOG_PDEBUG_START:   str = "  [DEBUG]  "
LOG_PDEBUG_END:     str = "\n"
LOG_PINFO_START:    str = "  [INFO ]  "
LOG_PINFO_END:      str = "\n"
LOG_PWARNING_START: str = "  [WARN ]  "
LOG_PWARNING_END:   str = "\n"
LOG_PERROR_START:   str = "  [ERROR]  "
LOG_PERROR_END:     str = "\n"

import io
debug: bool = False
logfile: io.TextIOWrapper | None = None

def _handle_logfile(msg: str) -> None:
    global logfile
    if logfile is not None:
        logfile.write(msg)
        logfile.flush()
    return None

def set_logfile(filename: str) -> None:
    global logfile
    logfile = open(filename, "w")
    return None

def get_logfile() -> io.TextIOWrapper | None:
    global logfile
    return logfile

def set_debug_mode(active: bool = True) -> None:
    global debug
    debug = active
    return None

def get_debug_mode() -> bool:
    global debug
    return debug

def pdebug(msg: str) -> None:
    global debug
    if debug:
        print(f"{PDEBUG_START}{msg}{PDEBUG_END}", end="")
        _handle_logfile(f"{LOG_PDEBUG_START}{msg}{LOG_PDEBUG_END}")
    return None

def pinfo(msg: str) -> None:
    print(f"{PINFO_START}{msg}{PINFO_END}", end="")
    _handle_logfile(f"{LOG_PINFO_START}{msg}{LOG_PINFO_END}")
    return None

def pwarning(msg: str) -> None:
    print(f"{PWARNING_START}{msg}{PWARNING_END}", end="")
    _handle_logfile(f"{LOG_PWARNING_START}{msg}{LOG_PWARNING_END}")
    return None

def perror(msg: str) -> None:
    print(f"{PERROR_START}{msg}{PERROR_END}", end="")
    _handle_logfile(f"{LOG_PERROR_START}{msg}{LOG_PERROR_END}")
    return None