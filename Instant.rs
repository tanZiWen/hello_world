use std::time::Instant;

fn main() {
    // 获取当前时间戳
    let start = Instant::now();

    // 需要统计执行时间的代码块
    let sum: u64 = (1..1000000).sum();
    
    println!("The sum is: {}", sum);

    // 获取执行结束的时间戳
    let duration = start.elapsed();

    // 打印执行时长，单位为秒和纳秒
    println!("Execution time: {:?}", duration);
}

