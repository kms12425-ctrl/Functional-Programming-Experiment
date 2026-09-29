(* 任务：编写奇数判断函数 oddP : int -> bool，当且仅当该数为奇数时返回 true。
   要求：不能调用函数 evenP 或 mod。
   oddP : int -> bool
   REQUIRES: n >= 0
   ENSURES: oddP n evaluates to true iff n is odd. *)

(* 输出一个bool *)
fun printBool (a:bool) = print(Bool.toString(a)^" "); 

(* 读取一个整数 *)
fun getInt () = Option.valOf (TextIO.scanStream (Int.scan StringCvt.DEC) TextIO.stdIn);
 
(* oddP : int -> bool *)
(* REQUIRES: n >= 0 *)
(* ENSURES: if n is odd *)
(*  编写函数oddP, 完成Begin和End间代码的修改  *)   

(****   Begin      ****)
fun oddP (0) = false
| oddP (1) = true
| oddP n = oddP (n-2)
(****   End      ****)

val m = getInt ();
printBool (oddP m); 
