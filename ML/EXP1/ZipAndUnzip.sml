(* 任务：
   (1) zip : string list * int list -> (string * int) list
       提取第一个 string list 的第 i 个元素和第二个 int list 的第 i 个元素组成二元组，
       结果长度为两个参数 list 长度的最小值。
   (2) unzip : (string * int) list -> string list * int list
       执行 zip 的反向操作，将二元组 list 分解为两个 list。
   思考：对所有 L1: string list 和 L2: int list，unzip(zip(L1, L2)) = (L1, L2) 是否成立？
         当 L1 和 L2 长度相同时成立；长度不同时，zip 会截断较长 list，
         unzip 无法恢复被截断的元素，故等式不成立。 *)

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
 (* zip : string list * int list -> (string * int) list *)
 fun zip ([], _) = []
   | zip (_, []) = []
   | zip (s::ss, i::ii) = (s, i) :: zip(ss, ii);

 (* unzip : (string * int) list -> string list * int list *)
  fun unzip ([]) = ([], [])
    | unzip ((s, i)::xs) =
        let
            val (ss, ii) = unzip(xs)
        in
            (s::ss, i::ii)
        end;
(*** End ***)

val test1 = [("a",1), ("b",2)] = zip(["a","b"],[1,2]);
print (Bool.toString test1 ^ "\n");

val test2 = [("a",1)] = zip(["a"],[1,2,3]);
print (Bool.toString test2 ^ "\n");

val test3 = [("a",1), ("b",2)] = zip(["a","b","c","d"],[1,2]);
print (Bool.toString test3 ^ "\n");

val test4 = (["a","b"],[1,2]) = unzip([("a",1), ("b",2)]);
print (Bool.toString test4 ^ "\n");

val test5 = (["dragon","llama","muffin"],[42,54,76]) =
              unzip([("dragon",42),("llama",54),("muffin",76)]);
print (Bool.toString test5 ^ "\n");
