include gmsl

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

override CMD = $(if $(call gte,$(VERBOSE),$(1)),$(2),@echo "$(3)"; $(2))

# Example use:
# $(call MAYBEPRINT,THRESHOLD)
# If THRESHOLD is less than or equal to VERBOSE, then this expands to nothing (so we print the command).
# Else, it expands to @.
override MAYBEPRINT = $(if $(call lte,$(1),$(VERBOSE)),,@)

override MK_LOWERCASE = $(shell echo $1 | tr '[:upper:]' '[:lower:]')

DESKTOPENV ?= $(shell echo $$XDG_CURRENT_DESKTOP)

ifeq ($(true),$(call seq,$(call MK_LOWERCASE,$(DESKTOPENV)),gnome))
	override DESKTOPENV := gnome
	override SCREENRES := $(shell xrandr | sed -n -e 's/.*current \([0-9]*\) x \([0-9]*\),.*/\1 \2/p')
	override __OPEN_TWO_TERMINALS_width := $(shell echo $(SCREENRES) | awk '{print $$1/2}')
	override __OPEN_TWO_TERMINALS_height := $(shell echo $(SCREENRES) | awk '{print $$2}')
	override __OPEN_TWO_TERMINALS_x_first := 0
	override __OPEN_TWO_TERMINALS_y_first := 0
	override __OPEN_TWO_TERMINALS_x_second := $(shell echo $(SCREENRES) | awk '{print $$1/2}')
	override __OPEN_TWO_TERMINALS_y_second := 0
	override define OPEN_TWO_TERMINALS =
		@gnome-terminal --window --title=$(call first,$(1)) \
			--geometry=+$(__OPEN_TWO_TERMINALS_screenpos_first) \
			-- bash -c "$(call first,$(call rest,$(1)))" & \
		gnome-terminal --window --title=$(call first,$(2)) \
			--geometry=+$(__OPEN_TWO_TERMINALS_screenpos_second) \
			-- bash -c "$(call first,$(call rest,$(2)))"
	endef
else ifeq $(call seq,$(call MK_LOWERCASE,$(DESKTOPENV)),kde)
	override DESKTOPENV := kde
else ifeq $(call seq,$(DESKTOPENV),)
	$(call PERROR,Desktop environment not detected)
else
	$(call PERROR,Desktop environment '$(DESKTOPENV)' not supported)
endif