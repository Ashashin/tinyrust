; TinyRAM V=1.000 W=63 K=8
;;; Collatz
_init:
read r0, 0                  ; Read an entry value from a tape file
mov r4, 0                   ; init the iteration counter to 0 

_loop:                      
    cmpe r0, 1              ; Loop out criteria
    cjmp _end               ; 
    mov  r1, r0             ;
    umod r2, r1, 2          ; check if r1 is even
    cmpe r2, 0              ;
    cjmp _even              ;
    jmp  _odd               ;

_even:                      
    shr r0, r1, 1           ;  r0 divise by 2
    add r4, r4, 1           ;
    jmp _loop               ;

_odd:                                     
    mull r3, r1, 3          ; r3 = r0 * 3
    add  r3, r3, 1          ; r3++ 
    mov  r0, r3             ;
    add  r4, r4, 1          ;
    jmp  _loop              ;

_end:  
    store  r0, r4           ;
    answer 0             