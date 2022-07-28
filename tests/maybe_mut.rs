use mwt::{maybe_mut, mwt};

#[test]
fn simple_test() {
    let a = 3;
    let a_ref = my_function(&a);

    let mut b = 5;
    let b_mut = my_mut_function(&mut b);

    *b_mut += 7;

    let c = *a_ref + b;

    assert_eq!(c, 15);
}

#[test]
fn opt_test() {
    assert_eq!(my_opt_function(None), None);

    let mut u = 3u64;

    let o = Some(&mut u);
    let o = my_opt_mut_function(o);

    let v = *(o.unwrap());

    assert_eq!(v, 6u64);
    assert_eq!(u, 6u64);
}

#[test]
fn struct_test() {
    let a = SomeStruct {
        id: 0,
        a_vector: vec![],
    };
    let b = SomeStruct {
        id: 1,
        a_vector: vec![],
    };
    let s = SomeStruct {
        id: 2,
        a_vector: vec![a, b],
    };
    assert_eq!(s.my_accessor().id(), 1);
    let mut s = s; // following line is an error otherwise
    assert_eq!(s.my_mut_accessor().id(), 0);

    assert_eq!(s.my_always_mut_fn().id(), 0);
    assert_eq!(s.id(), 12);
    assert_eq!(s.my_always_mut_fn_mut().id(), 0);
    assert_eq!(s.id(), 22);

    for c in s.children_mut() {
        c.id += 1;
    }

    let mut children = s.children().iter().map(|c| c.id());
    assert_eq!(children.next(), Some(1));
    assert_eq!(children.next(), Some(2));
    assert_eq!(children.next(), None);
}

#[mwt]
fn my_mwt_function(val: &Mwt<i32>) -> &Mwt<i32> {
    val
}

#[mwt]
fn my_opt_mwt_function(val: Option<&Mwt<u64>>) -> Option<&Mwt<u64>> {
    if let Some(v) = val {
        #[if_mut]
        {
            *v *= 2;
        }
        Some(v)
    } else {
        None
    }
}
struct SomeStruct {
    id: usize,
    a_vector: Vec<SomeStruct>,
}

impl SomeStruct {
    fn id(&self) -> usize {
        self.id
    }
    #[maybe_mut]
    fn my_maybe_mut_accessor(&mut self) -> &MaybeMut<SomeStruct> {
        let mut a = 0;
        a += 1;
        #[if_mut]
        {
            let b = 0;
        }
        #[not_mut]
        {
            let b = 1;
        }
        a -= 1;
        // a == 0
        self.a_vector.get_maybe_mut(b + a).unwrap()
    }
    #[mwt(ignore_self)]
    fn my_always_mut_fn_mwt(&mut self) -> &Mwt<SomeStruct> {
        self.id += 10;
        self.a_vector.get_mwt(0).unwrap()
    }
    #[mwt]
    fn children_mwt(&mut self) -> &Mwt<Vec<SomeStruct>> {
        &mwt(self.a_vector)
    }
}

#[test]
fn return_type_test() {
    let result = get_guard_mut();
    assert_eq!(GuardTypeMut {}, result);

    let result = get_guard();
    assert_eq!(GuardType {}, result);

    let result = get_struct_mut();
    assert_eq!(StructA {}, result);

    let result = get_struct();
    assert_eq!(StructB {}, result);
}

#[derive(PartialEq, Debug)]
struct GuardType {}

#[derive(PartialEq, Debug)]
struct GuardTypeMut {}

#[mwt]
fn get_guard_mwt() -> GuardTypeMwt {
    GuardTypeMwt {}
}

#[derive(PartialEq, Debug)]
struct StructA {}

#[derive(PartialEq, Debug)]
struct StructB {}

#[mwt]
fn get_struct_mwt() -> MwtAlt<StructA, StructB> {
    #[if_mut]
    {
        StructA {}
    }
    #[not_mut]
    {
        StructB {}
    }
}
