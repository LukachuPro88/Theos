[BITS 16]
[ORG 0x7c00]

start:
  ; Zero segment registers
  xor ax, ax
  mov ds, ax
  mov es, ax
  mov ss, ax
  mov sp, 0x7c00

  ; Print 'T' Using BIOS interrupt
  mov ah, 0x0e
  mov al, 'T'
  int 0x10

  hlt

times 510 - ($ - $$) db 0
dw 0xaa55