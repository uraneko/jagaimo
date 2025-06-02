<h1>jagaimo</h1> 
jagaimo is a cli utilities library

### ToC
* Features
* Usage
* Benchmarks
* License

#### Features 
* auto type tree generation for the cli tool from the rules passed to the macro
* auto implementation of Help trait for all types in the cli type tree
* auto implementation of Lex trait for all types in the cli type tree
* auto implementation of Parse trait for all types in the cli type tree
* auto error generation when passed args don't match their defined types 

#### Usage 

add to your project with
```bash
cargo add jagaimo --features macro
```

then call the jagaimo macro in your project
```rust
jagaimo {
    // cli rules go here 
}
```
for more on this, head to <a href="examples"></a>


#### Benchmarks 
| bench | lex | parse | 
| --:-- | -:- | --:-- |
| auto gen and sed bench results here from the cargo bench command |

#### License 
Licensed under <a href = "LICENSE">MIT</a>

