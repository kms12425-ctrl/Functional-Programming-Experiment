(* 任务：定义函数 divisibleByThree : int -> bool，当 n 为 3 的倍数时返回 true，否则返回 false。
   要求：不能使用取余函数 mod。
   divisibleByThree : int -> bool
   REQUIRES: n >= 0
   ENSURES: divisibleByThree n evaluates to true if n is a multiple of 3 and to false otherwise. *)

(* 输出一个bool *)
fun printBool (a:bool) = print(Bool.toString(a)^" "); 

(* 读取一个整数 *)
fun getInt () = Option.valOf (TextIO.scanStream (Int.scan StringCvt.DEC) TextIO.stdIn);

(*  编写函数divisibleByThree, 完成Begin和End间代码的修改  *)    
(* divisibleByThree : int -> bool 	*)
(* REQUIRES: n>=0				*)
(* ENSURES: divisibleByThree n evaluates to true if n is a multiple of 3 and to false otherwise *)

(*****Begin*****)
fun divisibleByThree (0) = true
| divisibleByThree (1) = false
| divisibleByThree (2) = false
| divisibleByThree n = divisibleByThree (n-3)

(*****End*****)

val m = getInt ();
printBool (divisibleByThree m);
