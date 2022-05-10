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

 they also have a `mwt()` and `maybe_mut()` function* respectively for things like `return &mwt(self.0)`

 both also let you pass an argument `ignore_self` e.g. `#[mwt::maybe_mut(ignore_self)]` to stop mwt from messing with the `&self` (or `&mut self`) parameter. stripping `mut` from `&mut self` is the default because taking `&T<self>` is a parse error, and most of the time this is the desired behavior (at least for my use cases).

 there isn't currently a way to handle functions of the form `_ref`/`_mut` but one may be added in the future (maybe `rwf` which becomes either `ref` or `mut`?)


 *Not actually a function, but the proc macro looks for it being used like a function call

## Example:

mwt lets you write:

```Rust
use mwt::mwt;

struct SomeStruct {
    a_vector: Vec<SomeStruct>,
}

impl SomeStruct {
    #[mwt]
    fn my_mwt_accessor(&mut self) -> &Mwt<SomeStruct> {
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
        let i = 0;
    }
    #[not_mut] {
        // code for only the non-mut version of the function
        let i= 1;
    }
    // do something with i
    self.get_mwt_flag_by_index(i)
}
```

Basically write the mutable version of your function, but for identifiers, replace `mut` with `mwt` and for types replace `&mut T` with `&Mwt<T>`

Alternatively you can use `mwt::maybe_mut` if you feel that's more readable. example:

```Rust
#[mwt::maybe_mut]
pub fn get_maybe_mut<T: 'static + Component>(&mut self) -> Option<&MaybeMut<T>> {
    // use #[if_mut]{} and #[not_mut]{} for conditional behavior
    //      or for cases where mwt isn't powerful enough yet
    //      like .as_ref() vs .as_mut()
    #[if_mut] { println!("if_mut"); }
    #[not_mut] { println!("not_mut"); }
    //   use &MaybeMut<T> for &mut types
    let map: &MaybeMut<HashMap<TypeId,Box<dyn Component>>>
    //    and &maybe_mut(...) for taking mut references
            = &maybe_mut(self.components);
    // use _maybe_mut_ in function calls, etc.
    map.get_maybe_mut(&TypeId::of::<T>())
        .and_then(|c| c.cast_maybe_mut::<T>())
}
```
results in two functions:
```Rust
pub fn get_<T: 'static + Component>(& self) -> Option<&T> {
    println!("not_mut"); 
    let map: &HashMap<TypeId,Box<dyn Component>> = &self.components;
    map.get(&TypeId::of::<T>()).and_then(|c| c.cast::<T>())
}
pub fn get_mut<T: 'static + Component>(&mut self) -> Option<&mut T> {
    println!("if_mut");
    let map: &mut HashMap<TypeId,Box<dyn Component>> = &mut self.components;
    map.get_mut(&TypeId::of::<T>()).and_then(|c| c.cast_mut::<T>())
}
```

---
## What's it actually doing?

`mwt::mwt` basically just replaces the function with two copies (i.e. a non-mut and mut version) and does a few things on those:

 - replace any occurrences of type references like `&Mwt<T>` with `&T` and `&mut T` respectively
 - replace any occurences of `mwt(expr)` with `expr` and `mut expr` respectively
 - for the non-mut version of the function:
    - it takes all identifiers it finds and trims any starting "mwt\_" and ending "\_mwt" and replaces "\_mwt\_" with "\_"
    - it takes all types it finds and removes any instances of "Mwt"
 - for the mut version of the function:
    - it takes all identifiers it finds and replaces any instances of "mwt" with "mut"
    - it takes all types it finds and replaces any instances of "Mwt" with "Mut"
 - to allow for other ways behavior can differ, the mut version strips any occurences of `#[not_mut]{...}` and the non-mut version strips any occurrences of `#[if_mut]{...}` (the ones that aren't stripped have their braces removed, so be aware of that)
 - to allow for differing types, `MwtAlt<First, Second>` is replaced with either `First` or `Second` in the mut and non-mut versions respectively

 `mwt::maybe_mut` is identical just with different strings.
 (`Mwt` -> `MaybeMut`, `mwt` -> `maybe_mut`, `MwtAlt` -> `MutOrElse`)


---
## Found a bug? Need a feature?

Please file an issue or submit a pull request!
