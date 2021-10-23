; TinyRAM V=1.000 W=63 K=8
;;; Collatz
_init:
mov r0, 32768               ; mettre la valeur qu'on veut tester, ici j'ai pris 32768
mov r4, 0                   ; chargement du compteur d'itérations à zéro

_loop:                      
    cmpe r0, 1              ; calcul du critère d'arrêt de la boucle (on arrive sur la suite 4,2,1 )
    cjmp _end               ; 
    mov  r1, r0              ;
    umod r2, r1, 2          ; on checke si r0 est pair
    cmpe r2, 0
    cjmp _even              ;
    jmp  _odd               ;

_even:                      
    shr r0, r1, 1           ; on divise r0 par 2
    add r4, r4, 1           ;
    jmp _loop               ;

_odd:                                     
    mull r3, r1, 3          ; r3 = r0 * 3
    add  r3, r3, 1           ; r3++ 
    mov  r0, r3              ;
    add  r4, r4, 1           ;
    jmp  _loop               ;

_end:  
    store  r0, r4 
    answer 0             
