fn main() {
    let x = String::from("Hello");

    // 闭包使用 `move` 来捕获 `x`，这意味着 `x` 的所有权被转移到闭包中
    let consume = move || {
        println!("{}", x);  // 闭包消费 `x`
    };

    consume();  // 这里调用闭包，`x` 的所有权已被转移，无法再使用 `x`
    // println!("{}", x); // 错误：`x` 的所有权已经被移动
}
