.altmacro
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm

.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm

    .section .text.trampoline
    .globl __alltraps
    .globl __restore
    .align 2

# 1. point sp to trap context(original user stack is saved in sscratch) -> for step 2
# 2. save register values(including user stack) to trap context
# 3. switch to kernel stack and kernel memory space(stored data in trap context)
# 4. call trap_handler
__alltraps:
    # see __restore: `csrw sscratch, a0` which put kernel stack value(beginning of trap context) to sscratch
    csrrw sp, sscratch, sp
    # now sp->*TrapContext in user space, sscratch->user stack
    # save other general purpose registers
    sd x1, 1*8(sp)
    # skip sp(x2), we will save it later
    sd x3, 3*8(sp)
    # skip tp(x4), application does not use it
    # save x5~x31
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr
    # we can use t0/t1/t2 freely, because they have been saved in TrapContext
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    # read user stack from sscratch and save it in TrapContext
    csrr t2, sscratch
    sd t2, 2*8(sp)
    # load kernel_satp into t0
    ld t0, 34*8(sp)
    # load trap_handler into t1
    ld t1, 36*8(sp)
    # move to kernel_sp
    ld sp, 35*8(sp)
    # switch to kernel space
    csrw satp, t0
    sfence.vma
    # jump to trap_handler
    jr t1

# __restore(trap_context: *TrapContext, user_token: usize)
# 1. switch to user space(but still in supervisor mode)
# 2. point sp to trap context
# 3. restore register values in trap context
# 4. restore sp to user stack
# 5. sret
__restore:
    # a0: *TrapContext in user space(Constant); a1: user space token
    # switch to user space
    csrw satp, a1
    sfence.vma
    csrw sscratch, a0
    mv sp, a0
    # now sp points to TrapContext in user space, start restoring based on it
    # restore sstatus/sepc
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    # restore general purpose registers except x0/sp/tp
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    # back to user stack
    ld sp, 2*8(sp)
    sret
