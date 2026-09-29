(* 任务：给定数组 A[1..n]，前缀和 PrefixSum[i] = A[0]+A[1]+...+A[i-1]。
   (1) PrefixSum : int list -> int list，要求 WPrefixSum(n) = O(n^2)。
   (2) fastPrefixSum : int list -> int list，要求 WfastPrefixSum(n) = O(n)。
   示例：PrefixSum [] = []
         PrefixSum [5,4,2] = [5, 9, 11]
         PrefixSum [5,6,7,8] = [5, 11, 18, 26] *)

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

(*  完成Begin和End间代码 *)    
(*****Begin*****)
fun PrefixSum [] = []
  | PrefixSum (x::xs) = x :: (map (fn y => x + y) (PrefixSum xs))

fun fastPrefixSum L =
    let
        fun helper ([], _, acc) = rev acc
          | helper (x::xs, sum, acc) =
            let
                val current = sum + x
            in
                helper (xs, current, current :: acc)
            end
    in
        helper (L, 0, [])
    end
(*****End*****)

val L = getIntList(3);
printIntList (PrefixSum L);
printIntList (fastPrefixSum L);