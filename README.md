# mwt

## Hey! You! Read this before using!

### mwt was thrown together pretty quickly for personal use, because I couldn't find an existing crate that does this.
### There are probably bugs, there are definitely plenty of edge cases that haven't been considered, and the error messages are rather poor.
### It'll probably get better as I use it and fix issues I find, but caveat emptor or whatever

---

Generate mut and non-mut versions of the same function without duplicating code!

mwt provides two mostly identical macros: `mwt` and `maybe_mut`
 - `mwt` looks for `mwt` in identifiers, and looks for types like `&Mwt<T>`
 - `maybe_mut` does the same for `maybe_mut` and `&MaybeMut<T>`

 both let you put `#[if_mut]` and `#[not_mut]` before blocks to have conditionally present sections.

 they also have a `mwt()` and `maybe_mut()` macro respectively for things like `return &mwt(self.0)`

 both also let you pass an argument `ignore_self` e.g. `#[mwt::maybe_mut(ignore_self)]` to stop mwt from messing with the `&self` (or `&mut self`) parameter. stripping `mut` from `&mut self` is the default because takeing `&T<self>` is a parse error, and most of the time this is the desired behavior (at least for my use cases).

 there isn't currently a way to handle functions of the form `_ref`/`_mut` but one may be added in the future (maybe `rwf` which becomes either `ref` or `mut`?)

## Example:

mwt lets you write:

```Rust
use mwt::mwt;

struct SomeStruct {
    a_vector: Vec<SomeStruct>,
}

impl SomeStruct {
    #[mwt]
    fn my_mwt_accessor(&mut self) -> &Mwt(SomeStruct) {
        let mut a = 0;
        a = a + 1;
        let b = &mwt(a);
        #[if_mut] {
            println!("Hello from my_mut_accessor()!");
        }
        #[not_mut] {
            println!("Hello from my_accessor()!");
        }
        self.a_vector.get_mwt(0).unwrap()
    }
}
```

which results in two functions:

```Rust
impl SomeStruct {
    fn my_accessor(&self) -> &SomeStruct {
        let mut a = 0;
        a = a + 1;
        let b = &a;
        println!("Hello from my_accessor()!");
        self.a_vector.get(0).unwrap()
    }
    fn my_mut_accessor(&mut self) -> &mut SomeStruct {
        let mut a = 0;
        a = a + 1;
        let b = &mut a;
        println!("Hello from my_mut_accessor()!");
        self.a_vector.get_mut(0).unwrap()
    }
}
```
---
## How to use

e.g.

```Rust
#[mwt::mwt]
fn my_mwt_method(&'a mut self, other_param: i32) -> &Mwt<bool> {
    #[if_mut] {
        //code for only the mut version of the function
        let a = 0;
    }
    #[not_mut] {
        // code for only the non-mut version of the function
        let a = 1;
    }
    // do something with a
    self.get_mwt_flag_by_index(a)
}
```

Basically write the mutable version of your function, but for identifiers, replace `mut` with `mwt` and for types replace `&mut T` with `&Mwt<T>`

Alternatively you can use `mwt::maybe_mut` if you feel that's more readable.

---
## What's it actually doing?

`mwt::mwt` basically just replaces the function with two copies (i.e. a non-mut and mut version) and does a few things on those:

 - replace any occurrences of type references like `&Mwt<T>` with `&T` and `&mut T` respectively
 - replace any occurences of `mwt(expr)` with `expr` and `mut expr` respectively
 - for the non-mut version of the function, it takes all identifiers it finds and trims any starting "mwt\_" and ending "\_mwt" and replaces "\_mwt\_" with "\_"
 - for the mut version of the function, it takes all identifiers it finds and replaces any instances of "mwt" with "mut"
 - to allow for other ways behavior can differ, the mut version strips any occurences of `#[not_mut]{...}` and the non-mut version strips any occurrences of `#[if_mut]{...}` (the ones that aren't stripped have their braces removed, so be aware of that)

 `mwt::maybe_mut` is identical just with different strings.


---
## Found a bug? Need a feature?

Please file an issue or submit a pull request!
