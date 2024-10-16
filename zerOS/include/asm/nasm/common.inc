
%macro EXTERN_FUNC 1
    global %1
    align 16
%1:
%endmacro
