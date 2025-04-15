include gnumake/gmsl

override PINFO  = $(info [INFO ]  $(1))
override PWARN  = $(warning [WARN ]  $(1))
override PERROR = $(error [ERROR]  $(1))

override define CC_TUNE_FOR =
	-march=$1 -mtune=$1
endef

override define TRY_COMPILE =
	$(shell ((echo 'int main() { return 0; }' | $(1) $(2) -nostdlib -nostartfiles -x c -o /dev/null - 2>/dev/null); echo $$?))
endef

override CC_SUPPORTS_OPTION = $(call eq,$(call TRY_COMPILE,$(KCC),$(1) -c),0)

override REPEAT_CHAR = $(if $(call gt,$(2),0),$(1)$(call REPEAT_CHAR,$(1),$(call subtract,$(2),1)),)

override __CMD_MAXPREFIXLEN := 10
override __CMD_CALCULATE_SPACES = $(call subtract,$(__CMD_MAXPREFIXLEN),$(call strlen,$(1)))
override CMD = $(if $(call gte,$(VERBOSE),$(1)),$(2),@echo "$(3)$(call REPEAT_CHAR,$(space),$(call __CMD_CALCULATE_SPACES,$(3)))$(4)"; $(2))

override MAYBEDO = $(if $(call gte,$(VERBOSE),$(1)),$(2),@true)

override MK_OBJFILE = $(addprefix obj/,$(addsuffix .o,$(1)))
override MK_DEPFILE = $(addprefix obj/,$(addsuffix .d,$(1)))

# Example use:
# $(call MAYBEPRINT,THRESHOLD)
# If THRESHOLD is less than or equal to VERBOSE, then this expands to nothing (so we print the command).
# Else, it expands to @.
override MAYBEPRINT = $(if $(call gte,$(VERBOSE),$(1)),,@)

override MK_LOWERCASE = $(shell echo $1 | tr '[:upper:]' '[:lower:]')

DESKTOPENV ?= $(shell echo $$XDG_CURRENT_DESKTOP)

# OPEN_TWO_TERMINALS
# arg1: title1
# arg2: command1
# arg3: title2
# arg4: command2
# Opens two terminals with the specified titles and commands.

#ifeq ($(true),$(call seq,$(call MK_LOWERCASE,$(DESKTOPENV)),gnome))
#override DESKTOPENV := gnome
#override SCREENRES := $(shell xrandr | sed -n -e 's/.*current \([0-9]*\) x \([0-9]*\),.*/\1 \2/p')
#override __OPEN_TWO_TERMINALS_width := $(shell echo $(SCREENRES) | awk '{print $$1/2}')
#override __OPEN_TWO_TERMINALS_height := $(shell echo $(SCREENRES) | awk '{print $$2}')
#override __OPEN_TWO_TERMINALS_x_first := 0
#override __OPEN_TWO_TERMINALS_y_first := 0
#override __OPEN_TWO_TERMINALS_x_second := $(shell echo $(SCREENRES) | awk '{print $$1/2}')
#override __OPEN_TWO_TERMINALS_y_second := 0
#override define OPEN_TWO_TERMINALS =
#@gnome-terminal --window --title="$1" --working-directory=$(PWD) \
#--geometry=$(__OPEN_TWO_TERMINALS_width)x$(__OPEN_TWO_TERMINALS_height)+$(__OPEN_TWO_TERMINALS_x_first)+$(__OPEN_TWO_TERMINALS_y_first) \
#-- bash -c "$2" & \
#gnome-terminal --window --title="$3" --working-directory=$(PWD) \
#--geometry=$(__OPEN_TWO_TERMINALS_width)x$(__OPEN_TWO_TERMINALS_height)+$(__OPEN_TWO_TERMINALS_x_second)+$(__OPEN_TWO_TERMINALS_y_second) \
#-- bash -c "$4" &
#endef
#else ifeq ($(call seq,$(call MK_LOWERCASE,$(DESKTOPENV))),kde)
#override DESKTOPENV := kde
#override SCREENRES := $(shell xrandr | sed -n -e 's/.*current \([0-9]*\) x \([0-9]*\),.*/\1 \2/p')
#override __OPEN_TWO_TERMINALS_width := $(shell echo $(SCREENRES) | awk '{print $$1/2}')
#override __OPEN_TWO_TERMINALS_height := $(shell echo $(SCREENRES) | awk '{print $$2}')
#override __OPEN_TWO_TERMINALS_x_first := 0
#override __OPEN_TWO_TERMINALS_y_first := 0
#override __OPEN_TWO_TERMINALS_x_second := $(shell echo $(SCREENRES) | awk '{print $$1/2}')
#override __OPEN_TWO_TERMINALS_y_second := 0
#override define OPEN_TWO_TERMINALS =
#@konsole --new-tab --workdir $(PWD) --profile "Shell" --title "$1" --geometry $(__OPEN_TWO_TERMINALS_width)x$(__OPEN_TWO_TERMINALS_height)+$(__OPEN_TWO_TERMINALS_x_first)+$(__OPEN_TWO_TERMINALS_y_first) -e "$2" & \
#konsole --new-tab --workdir $(PWD) --profile "Shell" --title "$3" --geometry $(__OPEN_TWO_TERMINALS_width)x$(__OPEN_TWO_TERMINALS_height)+$(__OPEN_TWO_TERMINALS_x_second)+$(__OPEN_TWO_TERMINALS_y_second) -e "$4" &
#endef
#else ifeq ($(call seq,$(DESKTOPENV)),)
##$(call PERROR,Desktop environment not detected)
#else
#$(call PERROR,Desktop environment '$(DESKTOPENV)' not supported)
#endif

override __DBGSESSION_QEMU_CMD = $(KQEMU) $(KQEMU_RUNFLAGS) $1 -s
#override define START_DBG_SESSION =
#$(call OPEN_TWO_TERMINALS,(zerOS) QEMU,$(call __DBGSESSION_QEMU_CMD,$1),(zerOS) GDB,$(KGDB) $(basename $1))
#endef
override define START_DBG_SESSION =
$(call __DBGSESSION_QEMU_CMD,$1)
endef

comma :=,
space := $(__gmsl_space)
