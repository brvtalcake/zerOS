# If not already existing, create the directory for object files.
$(call PINFO,Creating object directory)
$(shell mkdir -p obj)

# If not already existing, create the directory for binary files.
$(call PINFO,Creating binary directory)
$(shell mkdir -p bin)