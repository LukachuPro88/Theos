[BITS 16]
[ORG 0x7e00]

; segment selector offsets
GDT_32_CODE_SEG equ 0x08
GDT_32_DATA_SEG equ 0x10
GDT_64_CODE_SEG equ 0x18  ; New 64-bit code selector

start:
    ; Zero segment registers and set up stack
    xor ax, ax
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov sp, 0x7c00

    mov si, msg_start
    call print

    ; Enable A20 via port 0x92
    in al, 0x92
    or al, 2
    out 0x92, al

    mov si, msg_a20
    call print

    mov si, msg_gdt
    call print

    cli                 ; disable interrupts before GDT
    lgdt [gdt_descriptor]

    ; Enter protected mode
    mov eax, cr0
    or eax, 1
    mov cr0, eax

    ; manually encoded 32-bit far jump: opcode 0x66 0xEA + 32-bit offset + 16-bit selector
    db 0x66
    db 0xea
    dd protected_mode
    dw GDT_32_CODE_SEG
    ; start ends here — execution never falls through

; -------- Print --------
print:
    mov ah, 0x0e
.loop:
    lodsb
    cmp al, 0
    je .done
    int 0x10
    jmp .loop
.done:
    ret
; ----------------

; -------- 32-bit Protected Mode --------
[BITS 32]
protected_mode:
    ; Reload segment registers with data selector
    mov ax, GDT_32_DATA_SEG
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    mov esp, 0x90000        ; new 32-bit stack

    ; -------- Page Tables --------
    ; Clear memory for page tables (0x1000 - 0x5000)
    mov edi, 0x1000         ; start of page table area
    mov cr3, edi            ; tell CPU where PML4 lives
    xor eax, eax
    mov ecx, 4096           ; clear 4096 * 4 bytes = 16KB
    rep stosd               ; fill with zeros
    mov edi, cr3            ; reset edi back to PML4

    ; PML4 entry → points to PDPT at 0x2000
    mov dword [edi], 0x2003         ; present + writable
    add edi, 0x1000

    ; PDPT entry → points to PD at 0x3000
    mov dword [edi], 0x3003         ; present + writable
    add edi, 0x1000

    ; PD entry → points to PT at 0x4000
    mov dword [edi], 0x4003         ; present + writable
    add edi, 0x1000

    ; PT entries → map first 2MB (512 pages of 4KB)
    mov ebx, 0x00000003             ; first page, present + writable
    mov ecx, 512                    ; 512 entries
.map_pt:
    mov dword [edi], ebx
    add ebx, 0x1000                 ; next 4KB page
    add edi, 8                      ; next PT entry
    loop .map_pt
    ; ----------------

    ; -------- Enter Long Mode --------
    ; Enable PAE (Physical Address Extension) — required for long mode
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    ; Set LME (Long Mode Enable) bit in EFER MSR
    mov ecx, 0xC0000080             ; EFER MSR number
    rdmsr                           ; Read current EFER status first
    or eax, 1 << 8                  ; set LME bit directly and safely
    wrmsr                           ; write it

    ; Enable paging — this activates long mode
    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax

    ; Far jump to 64-bit code, flushes pipeline
    jmp GDT_64_CODE_SEG:long_mode
    ; ----------------

; -------- 64-bit Long Mode --------
[BITS 64]
long_mode:
    ; Write 'L' to VGA to confirm we're in long mode
    mov rax, 0x0F4C0F4C0F4C0F4C     ; 'L' with white on black, repeated
    mov qword [0xB8000], rax

    cli
    hlt
; ----------------

; -------- GDT --------
align 8
gdt_start:

gdt_null:               ; entry 0 - required null descriptor
    dq 0

gdt_32_code:            ; entry 1 - 32-bit protected mode code segment
    dw 0xFFFF           ; limit low
    dw 0                ; base low
    db 0                ; base middle
    db 10011010b        ; access byte (present, ring 0, code, executable, readable)
    db 11001111b        ; flags + limit high (32-bit, 4KB granularity)
    db 0                ; base high

gdt_32_data:            ; entry 2 - data segment
    dw 0xFFFF           ; limit low
    dw 0                ; base low
    db 0                ; base middle
    db 10010010b        ; access byte (present, ring 0, data, writable)
    db 11001111b        ; flags + limit high
    db 0                ; base high

gdt_64_code:            ; entry 3 - 64-bit long mode code segment
    dw 0xFFFF           ; limit low
    dw 0                ; base low
    db 0                ; base middle
    db 10011010b        ; access byte (present, ring 0, code, executable, readable)
    db 10101111b        ; flags + limit high (64-bit, 4KB granularity)
    db 0                ; base high

gdt_end:

gdt_descriptor:
    dw gdt_end - gdt_start - 1     ; size of GDT minus 1
    dd gdt_start                    ; address of GDT

; ---------------

msg_start  db "Stage 2 loaded", 0x0D, 0x0A, 0
msg_a20    db "A20 enabled",    0x0D, 0x0A, 0
msg_gdt    db "GDT loaded",     0x0D, 0x0A, 0

times 1024 - ($ - $$) db 0