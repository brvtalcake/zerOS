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