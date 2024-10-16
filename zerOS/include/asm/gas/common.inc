# Assumes ATT syntax

.macro push16 reg
    pushw \reg
.endm

.macro pop16 reg
    popw \reg
.endm

.macro push64 reg
    pushq \reg
.endm

.macro pop64 reg
    popq \reg
.endm

.macro mov16 from, to
    movw \from, \to
.endm

.macro mov64 from, to
    movq \from, \to
.endm

.macro EXTERN_FUNC name
    .global \name
    .type \name, @function
    .balign 16
\name:
.endm