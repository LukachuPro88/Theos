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

  call load_second_stage

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

; Load second stage since 512 bytes isnt enough for the kernel
load_second_stage:
  mov ah, 0x02
  mov al, 2
  mov ch, 0
  mov cl, 2
  mov dh, 0
  mov dl, 0x80
  mov bx, 0x7e00
  int 0x13

  jc disk_error
  jmp 0x7e00

disk_error:
  mov si, disk_err_msg
  call print
  hlt

msg           db "Theos is booting...", 0x0D, 0x0A, 0
disk_err_msg  db "Disk read failed", 0x0D, 0x0A, 0

times 510 - ($ - $$) db 0
dw 0xaa55