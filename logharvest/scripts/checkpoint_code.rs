//! AI 关卡辅助：`Send` / `Sync` 的验证手段。
//!
//! # 为什么需要这个文件
//!
//! 脚手架里给了 `assert_send::<T>()`，但**没有给 `assert_sync`**。
//! 这是有意的，因为关卡第 2 步想让你自己发现一件事：
//!
//! > `thread::spawn` 只检查 `Send`，它根本不会帮你验证 `Sync`。
//!
//! 换句话说，如果你只靠「把类型 move 进线程里看编不编得过」来判断，
//! 你得到的结论只是「它是不是 `Send`」，而**对 `Sync` 一无所知**。
//! 这两件事必须用两套不同的手段分别验证。
//!
//! # 怎么用
//!
//! 把本文件的内容拷进 `src/main.rs`（或新建一个 `src/bin/checkpoint.rs`），
//! 然后 `cargo check`。哪一行报错，就说明那个结论不成立。
//!
//! 建议的做法是**逐行取消注释**，一次只放开一条，观察编译器的反应。

use logharvest::collector::{assert_send, LocalConfig};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

/// 把 `LocalConfig` 的 `Rc` 换成 `Arc`、`RefCell` 换成 `Mutex` 之后的版本。
///
/// 关卡第 3 步要求你构造这个类型并重新验证。
/// 注意：**不要直接抄这个定义**，先自己写一遍再对照。
pub struct SharedConfig {
    pub node_name: Arc<String>,
    pub cache: Mutex<Vec<String>>,
}

impl SharedConfig {
    pub fn new(node_name: impl Into<String>) -> SharedConfig {
        SharedConfig {
            node_name: Arc::new(node_name.into()),
            cache: Mutex::new(Vec::new()),
        }
    }
}

// ===========================================================================
// 下面这一整块是关卡的可执行部分。
// 把本文件作为 `src/bin/checkpoint.rs` 时，`cargo run --bin checkpoint`
// 会逐条打印结论；直接 `cargo check` 则会在编译期告诉你哪条不成立。
// ===========================================================================

fn main() {
    println!("=== 关卡：Send / Sync 验证 ===\n");

    // -----------------------------------------------------------------------
    // 手段一：验证 Send —— 用 thread::spawn
    // -----------------------------------------------------------------------
    //
    // 把一个值 move 进新线程。能过编译，就说明它是 Send。
    // 这是最直观的手段，但注意：**它只测 Send**。
    println!("[1] 用 thread::spawn 验证 Send");

    let cfg = LocalConfig::new("node-a");
    let h = std::thread::spawn(move || {
        // 只要这里能拿到 cfg，就说明 LocalConfig: Send 成立。
        println!("    LocalConfig 成功 move 进子线程 -> 它是 Send");
        cfg.node_name.len()
    });
    let _ = h.join();

    let shared = Arc::new(SharedConfig::new("node-a"));
    let shared2 = Arc::clone(&shared);
    let h = std::thread::spawn(move || {
        println!("    Arc<SharedConfig> 成功 move 进子线程 -> SharedConfig 是 Send");
        shared2.node_name.len()
    });
    let _ = h.join();

    // -----------------------------------------------------------------------
    // 手段二：验证 Sync —— 用「多线程共享引用」
    // -----------------------------------------------------------------------
    //
    // Sync 的定义是：&T 能安全地跨线程共享。
    // 所以要验证 Sync，必须真的把 &T 交给两个以上的线程。
    // thread::spawn 做不到这件事（它要 'static，收不了短生命周期的引用），
    // 所以这里用 std::thread::scope。
    println!("\n[2] 用 thread::scope 共享 &T 验证 Sync");

    let shared = SharedConfig::new("node-b");
    let shared_ref = &shared;
    std::thread::scope(|s| {
        for i in 0..2 {
            s.spawn(move || {
                // 两个线程同时持有 &SharedConfig。
                // 这能编过，说明 &SharedConfig 可以跨线程 -> SharedConfig: Sync
                let n = shared_ref.node_name.len();
                let mut c = shared_ref.cache.lock().unwrap();
                c.push(format!("thread-{}", i));
                n
            });
        }
    });
    println!("    &SharedConfig 被 2 个线程同时持有 -> 它是 Sync");

    // -----------------------------------------------------------------------
    // 手段三：编译期断言 —— 最严格、最省事
    // -----------------------------------------------------------------------
    //
    // 用泛型 bound 把结论钉死在类型系统里。
    // 如果结论不成立，编译直接失败，错误信息就是最干净的证据。
    println!("\n[3] 编译期断言");

    assert_send::<LocalConfig>();        // LocalConfig: Send      -> 成立
    assert_send::<SharedConfig>();       // SharedConfig: Send     -> 成立

    // 下面这一行如果放开注释，编译器会报错：
    //   `Rc<String>` cannot be sent between threads safely
    // 这就是「LocalConfig 不是 Send」的**直接证据**。
    //
    // assert_send::<Rc<String>>();

    // 下面这一行如果放开注释，编译器会报错：
    //   `RefCell<Vec<String>>` cannot be shared between threads safely
    // 注意错误信息里的措辞是 **shared**，不是 **sent** ——
    // 这正是 Sync 失败的措辞，与 Send 失败的措辞不同，值得仔细看。
    //
    // fn _assert_sync<T: Sync>() {}
    // _assert_sync::<LocalConfig>();

    println!("    已通过 assert_send::<LocalConfig>() 与 ::<SharedConfig>()");

    // -----------------------------------------------------------------------
    // 结论对照表（自己填空，写进实验报告）
    // -----------------------------------------------------------------------
    println!("\n=== 请你补全这张表 ===");
    println!("  类型              Send?   Sync?   验证手段");
    println!("  LocalConfig       ?       ?       ?");
    println!("  SharedConfig      ?       ?       ?");
    println!("\n提示：LocalConfig 的 Sync 结论，用手段二或手段三验不出来 ——");
    println!("      它会直接编译失败。那个失败信息就是你要粘贴的结论。");
}
