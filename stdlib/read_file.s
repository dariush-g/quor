	.file	"read_file.c"
	.text
	.section	.rodata.str1.1,"aMS",@progbits,1
.LC0:
	.string	"rb"
	.text
	.p2align 4
	.globl	read_file
	.type	read_file, @function
read_file:
.LFB22:
	.cfi_startproc
	pushq	%r12
	.cfi_def_cfa_offset 16
	.cfi_offset 12, -16
	leaq	.LC0(%rip), %rsi
	pushq	%rbp
	.cfi_def_cfa_offset 24
	.cfi_offset 6, -24
	pushq	%rbx
	.cfi_def_cfa_offset 32
	.cfi_offset 3, -32
	call	fopen@PLT
	testq	%rax, %rax
	je	.L2
	movl	$2, %edx
	xorl	%esi, %esi
	movq	%rax, %rbx
	movq	%rax, %rdi
	call	fseek@PLT
	movq	%rbx, %rdi
	call	ftell@PLT
	movq	%rbx, %rdi
	movq	%rax, %r12
	call	rewind@PLT
	leaq	1(%r12), %rdi
	call	malloc@PLT
	movq	%rax, %rbp
	testq	%rax, %rax
	je	.L10
	movq	%rax, %rdi
	movq	%rbx, %rcx
	movq	%r12, %rdx
	movl	$1, %esi
	call	fread@PLT
	movb	$0, 0(%rbp,%r12)
	movq	%rbx, %rdi
	call	fclose@PLT
.L1:
	movq	%rbp, %rax
	popq	%rbx
	.cfi_remember_state
	.cfi_def_cfa_offset 24
	popq	%rbp
	.cfi_def_cfa_offset 16
	popq	%r12
	.cfi_def_cfa_offset 8
	ret
.L10:
	.cfi_restore_state
	movq	%rbx, %rdi
	call	fclose@PLT
.L2:
	xorl	%ebp, %ebp
	jmp	.L1
	.cfi_endproc
.LFE22:
	.size	read_file, .-read_file
	.ident	"GCC: (GNU) 15.2.1 20250813"
	.section	.note.GNU-stack,"",@progbits
