.text
.global _start
.extern jump_to_kernel
.section .text.start, "ax"

.section .init.data, "aw"
.set KERNEL_BASE, 0xffff800000000000

.section .init.text, "ax"
.type _start, @function

.globl _start
_start:
    # Disable FFXSR and FastXeroxSave
    movq $0xc0000080, %rcx
    rdmsr
    andq $~0x4000, %rax
    wrmsr

    # Set CR0 to disable FPU exceptions
    movq %cr0, %rax
    orq $(1 << 18), %rax
    movq %rax, %cr0

    # Stack pointer is passed in RSI (multiboot tag)
    movq %rsi, %rsp

    # Enable SSE with OSFXSR and SSE
    movq %cr4, %rax
    orq $(1 << 9 | 1 << 10), %rax
    movq %rax, %cr4

    # Set EFER with NXE and LME
    movq $0xc0000080, %rcx
    rdmsr
    orq $(1 << 11 | 1 << 8), %rax
    wrmsr

    # Set PG, PE in CR0: enable protected mode + paging
    movq %cr0, %rax
    orq $(1 << 31 | 1 << 0), %rax
    movq %rax, %cr0

    # Set PML4 base in CR3 (0x1000 = identity 1:1 for top level)
    movq $0x1000, %rax
    movq %rax, %cr3

    # Set LMA in EFER (64-bit long mode active)
    movq $0xc0000080, %rcx
    rdmsr
    orq $1, %rax
    wrmsr

    jmp *%rdi

.size _start, . - _start
