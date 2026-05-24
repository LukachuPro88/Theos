[BITS 16]
[ORG 0x7e00]

start:
    mov si, msg_start
    call print

    ; Enable A20 via port 0x92
    in al, 0x92
    or al, 2
    out 0x92, al

    mov si, msg_a20
    call print

    cli                 ; disable interrupts before GDT
    lgdt [gdt_descriptor]

    mov si, msg_gdt
    call print

    ; Enter protected mode
    mov eax, cr0
    or eax, 1
    mov cr0, eax

    jmp CODE_SEG:protected_mode
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
    mov ax, DATA_SEG
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    mov esp, 0x90000        ; new 32-bit stack

    ; Can't use BIOS interrupts anymore so we write directly to VGA
    mov edi, 0xB8000        ; VGA text buffer address
    mov al, 'P'             ; character
    mov ah, 0x0F            ; white on black
    mov [edi], ax           ; write to screen

    cli
    hlt
; ----------------

; -------- GDT --------
[BITS 16]
gdt_start:

gdt_null:               ; entry 0 - required null descriptor
    dq 0

gdt_code:               ; entry 1 - code segment
    dw 0xFFFF           ; limit low
    dw 0                ; base low
    db 0                ; base middle
    db 10011010b        ; access byte (present, ring 0, code, executable, readable)
    db 11001111b        ; flags + limit high (32-bit, 4KB granularity)
    db 0                ; base high

gdt_data:               ; entry 2 - data segment
    dw 0xFFFF           ; limit low
    dw 0                ; base low
    db 0                ; base middle
    db 10010010b        ; access byte (present, ring 0, data, writable)
    db 11001111b        ; flags + limit high
    db 0                ; base high

gdt_end:

gdt_descriptor:
    dw gdt_end - gdt_start - 1     ; size of GDT minus 1
    dd gdt_start                    ; address of GDT

; segment selector offsets
CODE_SEG equ gdt_code - gdt_start  ; = 8
DATA_SEG equ gdt_data - gdt_start  ; = 16
; ---------------

msg_start  db "Stage 2 loaded", 0x0D, 0x0A, 0
msg_a20    db "A20 enabled",    0x0D, 0x0A, 0
msg_gdt    db "GDT loaded",     0x0D, 0x0A, 0

times 512 - ($ - $$) db 0