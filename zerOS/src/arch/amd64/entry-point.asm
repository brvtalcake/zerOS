BITS 64

; leaf 0x1
;; EDX
;;; do we need this one ? ;;;
;;; HAVE_CLFLUSH_EDX_MASK  equ 0x1 << 19 ;;;
HAVE_FXSAVE_EDX_MASK equ 0x1 << 24
HAVE_SSE_EDX_MASK    equ 0x1 << 25
HAVE_SSE2_EDX_MASK   equ 0x1 << 26
;; ECX
HAVE_SSE3_ECX_MASK  equ 0x1 <<  0
HAVE_SSSE3_ECX_MASK equ 0x1 <<  9
HAVE_SSE41_ECX_MASK equ 0x1 << 19
HAVE_SSE42_ECX_MASK equ 0x1 << 20
HAVE_XSAVE_ECX_MASK equ 0x1 << 26
HAVE_AVX_ECX_MASK   equ 0x1 << 28

; leaf 0x7, sub-leaf 0x0
;; EBX
HAVE_AVX2_EBX_MASK equ 0x1 <<  5

; leaf 0xd, sub-leaf 0x0
;; EAX
;; TODO: is that sufficient ?
HAVE_AVX512_EAX_MASK equ (0x1 << 5 | 0x1 << 6 | 0x1 << 7)

HAVE_FXSAVE  equ 0x1 <<  0
HAVE_XSAVE   equ 0x1 <<  1
HAVE_SSE     equ 0x1 <<  2
HAVE_SSE2    equ 0x1 <<  3
HAVE_SSE3    equ 0x1 <<  4
HAVE_SSSE3   equ 0x1 <<  5
HAVE_SSE41   equ 0x1 <<  6
HAVE_SSE42   equ 0x1 <<  7
HAVE_AVX     equ 0x1 <<  8
HAVE_AVX2    equ 0x1 <<  9
HAVE_AVX512  equ 0x1 << 10

%macro DECLARE_IMPORTED_SYM 1
    EXTERN %1
%endmacro

%macro DEFINE_EXTERN_SYM 2
    SECTION .%2
    ALIGN 16
    GLOBAL %1
%1:
%endmacro

%macro DEFINE_STATIC_SYM 2
    SECTION .%2
    ALIGN 16
    STATIC %1
%1:
%endmacro

%macro multipush 1-*
  %rep  %0
        push %1
  %rotate 1
  %endrep
%endmacro

%macro multipushw 1-*
  %rep  %0
        push WORD %1
  %rotate 1
  %endrep
%endmacro

%macro multipop 1-*
  %rep %0
  %rotate -1
        pop %1
  %endrep
%endmacro

%macro multipopw 1-*
  %rep  %0
        pop WORD %1
  %rotate 1
  %endrep
%endmacro

%define ON_STACK(=displacement, =size) %cond(%abs(size) % 8 == 0, [rsp + (%eval(displacement - 1) * %eval(%abs(size) / 8))],)

%macro if 1
    %push if
    j%-1  %$ifnot
%endmacro

%macro else 0
  %ifctx if
        %repl   else
        jmp     %$ifend
        %$ifnot:
  %else
        %error  "expected `if' before `else'"
  %endif
%endmacro

%macro endif 0
  %ifctx if
        %$ifnot:
        %pop
  %elifctx      else
        %$ifend:
        %pop
  %else
        %error  "expected `if' or `else' before `endif'"
  %endif
%endmacro

%macro zeroreg 1
    xor %1, %1
%endmacro

%macro cpuidleaf 1-2
    mov eax, %1
    %ifnempty %2
        mov ecx, %2
    %else
        zeroreg ecx
    %endif
%endmacro


DECLARE_IMPORTED_SYM zerOS_boot_setup

DEFINE_STATIC_SYM supported_features_buffer, bss
    resb 2

DEFINE_STATIC_SYM detect_features, text
    multipush rbx

    cpuidleaf 0x1
    multipush rax, rbx, rcx, rdx
    
    mov edx, ON_STACK(4, 32)

    multipop rax, rbx, rcx, rdx

    multipop rbx
    ret

DEFINE_EXTERN_SYM zerOS_entry_point, text
    ; preserve the potential parameters and the "preserved" registers
    multipush rdi, rsi, rdx, rcx, r8, r9, rbx, r12, r13, r14, r15

    ; do the initialization stuff
    ; ...

    ; pop registers
    multipop rdi, rsi, rdx, rcx, r8, r9, rbx, r12, r13, r14, r15

    ; call the real entry point
    ; TODO: maybe we should jump instead ?
    call zerOS_boot_setup
