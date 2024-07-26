
define PRINTVAR_RULE =
.PHONY: print_$(1)
print_$(1):
	@echo $($(1))
endef