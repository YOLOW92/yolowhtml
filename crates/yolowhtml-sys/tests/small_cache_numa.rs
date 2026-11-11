//! 同一组压测，但假装每个 CPU 是一个 NUMA 节点、线程缓存上限只有 4 KB：线程一迁移就换分片，
//! 块在分片之间流动，超额回收也时时触发。没有多节点机器时用它验证分片和回收的路径。

mod stress;

#[global_allocator]
static ALLOC: yolowhtml_sys::SmallCache = yolowhtml_sys::SmallCache;

fn setup() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| yolowhtml_sys::small_cache_simulate_numa(4096));
}
