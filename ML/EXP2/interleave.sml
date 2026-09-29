(* 任务：编写函数 interleave : int list * int list -> int list，
   合并两个 int list，元素交替出现，直至其中一个 list 结束，
   另一个 list 的剩余元素直接附加至结果尾部。
   示例：interleave([2], [4]) = [2, 4] *)

fun printInt (a:int) =
    print(Int.toString(a)^" ");

fun getInt () =
    Option.valOf (TextIO.scanStream (Int.scan StringCvt.DEC) TextIO.stdIn);
    
fun printIntList ( [] ) = ()
  | printIntList ( x::xs ) = 
    let
	val tmp = printInt(x)
    in
	printIntList(xs)
    end;

fun getIntList ( 0 ) = []
  | getIntList ( N:int) = getInt()::getIntList(N-1);

 
(*** Begin ***)
(*** Begin ***)

(* interleave: int list * int list -> int list *)
fun interleave ([], []) = []
| interleave ([], L2) = L2
| interleave (L1, []) = L1
| interleave (x::xs, y::ys) = x :: y :: interleave(xs, ys)

(*****End*****)


(*****End*****)

val L = getIntList(2);
val R = getIntList(6);
val O = interleave (L, R);
printIntList(O); 
