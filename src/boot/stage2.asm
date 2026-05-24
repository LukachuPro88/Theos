[BITS 16]
[ORG 0x7e00]

start: 
  mov si, msg_start
  call print

  ; Enable A20 via port 9x92
  in al, 0x92
  or al, 2
  out 0x92, al

  mov si, msg_a20
  call print

  cli
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

msg_start db "Stage 2 loaded", 0x0D, 0x0A, 0
msg_a20   db "A20 enabled", 0x0D, 0x0A, 0

times 512 - ($ - $$) db 0