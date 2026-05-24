[BITS 16]
[ORG 0x7c00]

start:
  ; Zero segment registers
  xor ax, ax
  mov ds, ax
  mov es, ax
  mov ss, ax
  mov sp, 0x7c00

  ; Print the message
  mov si, msg
  call print

  hlt

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

msg db "Theos is booting...", 0

times 510 - ($ - $$) db 0
dw 0xaa55