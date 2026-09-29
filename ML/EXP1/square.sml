(* 任务：编写递归函数 square 实现整数平方的计算，即 square n = n * n。
   要求：可调用函数 double，但不能使用整数乘法（*）运算。
   double : int -> int
   REQUIRES: n >= 0
   ENSURES: double n evaluates to 2 * n. *)

fun printInt (a:int) = print(Int.toString(a)^" ");

fun getInt () = Option.valOf (TextIO.scanStream (Int.scan StringCvt.DEC) TextIO.stdIn);


(*  完成Begin和End间代码的修改  *)    

(*****Begin*****)

(* double : int -> int *)
(* REQUIRES: n >= 0 *)
(* ENSURES: double n evaluates to 2 * n.*)

fun double (0 : int) : int = 0
    | double n = 2 + double (n - 1)

(* 编写函数square*)

(* square : int -> int *)
(* REQUIRES: n >= 0 *)
(* ENSURES: square n returns n * n *)

fun square (0 : int) = 	0
 | square n =  (square (n-1)) + (double n) - 1

(*****End*****)


val n = getInt();
printInt(square n);
