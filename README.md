The code is the repository accompanies [this article on my blog](http://www.nickwilcox.com/blog/arm_atomics/).

To run the version that performs correctly on X86 CPU's only use

```
cargo run --bin x86_only --release
```


To run the version that is correct on both CPU's use
```
cargo run --bin arm_and_x86 --release
```