fn main() {
    println!("cargo::rustc-check-cfg=cfg(value_repr, values(\"nan\", \"union\"))");
    println!("cargo::rustc-cfg=value_repr=\"nan\"");
}
