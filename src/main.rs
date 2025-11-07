mod union_find;
mod site;

use union_find::UnionFind;
use site::{Sites};
use std::{sync::{LazyLock, Mutex}};

/// Global UF tracking value interaction sets.
/// Whenever two values are observed interacting, union together the tags (SetIds)
/// stored in this structure.
/// Any time a new value is used (either literal values, or a new value is bound to
/// a new variable with `let`), make a new set in this structure with a unique SetId
/// to represent the value.
static VALUE_UF: LazyLock<Mutex<UnionFind<Tag<u32>>>> = LazyLock::new(|| Mutex::new(UnionFind::new()));

/// Tracks (potentially multiple) program sites which are under analysis.
/// At the beginning of a site, create a new Site using `site_ufs.get_site(id)`.
/// At the end of a site, call `site_ufs.get_site(id).update(value_uf)`.
/// View `site.rs` for more information.
static SITE_UFS: LazyLock<Mutex<Sites<Tag<u32>>>> = LazyLock::new(|| Mutex::new(Sites::new()));

fn main() {
    // simple_func(10, 100);
    // simple_func(20, 200);

    // without this line, we should see two abstract type sets in the output
    // due to the conditional on line 118, otherwise all variables will be in the same sets
    // simple_func(30, 300); 

    let res = complex_func(5);
    println!("{:?}", res);

    let site_ufs = SITE_UFS.lock().unwrap();
    site_ufs.print_analysis();
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
struct Tag<T> where T: Copy {
    addr: String,
    value: T,
}

impl<T> Tag<T> where T: Copy {
    fn new(value: &T) -> Self {
        Tag {
            addr: format!("{:p}", value),
            value: *value,
        }
    }
}

/// This is an example of a function that we want to analyze
/// Each white space seperated line is a line of code we are analyzing,
/// the first of which is the actual code, the following are the added
/// lines to perform ATI.
fn simple_func(x: u32, y: u32) -> u32 {
    let mut value_uf = VALUE_UF.lock().unwrap();
    let mut site_ufs = SITE_UFS.lock().unwrap();
    let site = site_ufs.get_site(0);  // create a new analysis site. View site.rs for more info

    value_uf.make_set(Tag::new(&x));  // for parameter input
    value_uf.make_set(Tag::new(&y));  // for parameter input
    site.observe_var("VAR:x".into(), Tag::new(&x));
    site.observe_var("VAR:y".into(), Tag::new(&y));

    let a: u32 = 2;
    value_uf.make_set(Tag::new(&a));
    site.observe_var("VAR:a".into(), Tag::new(&a));

    let b: u32 = 2;
    value_uf.make_set(Tag::new(&b));
    site.observe_var("VAR:b".into(), Tag::new(&b));

    value_uf.union(&Tag::new(&a), &Tag::new(&x));
    let result = a + x;
    value_uf.make_set(Tag::new(&result));
    value_uf.union(&Tag::new(&result), &Tag::new(&a));
    site.observe_var("VAR:result".into(), Tag::new(&result));

    value_uf.union(&Tag::new(&b), &Tag::new(&y));
    let test = b + y;
    value_uf.make_set(Tag::new(&test));
    value_uf.union(&Tag::new(&test), &Tag::new(&b));
    site.observe_var("VAR:test".into(), Tag::new(&test));

    if test > 300 {
        // TODO: THIS IMPLEMENTAITON REQUIRES SSA FORM!
        // is that fine ??? a smarter choice of tag probably addresses this.
        // problem comes from needing to register an interaction between
        // the value here in result, and the value stored in result2,
        // in otherwords, we cannot get rid of the old value, as we need to retain
        // it's tag for proper merging
        value_uf.union(&Tag::new(&result), &Tag::new(&test));
        let result2 = result + test;
        value_uf.make_set(Tag::new(&result2));
        value_uf.union(&Tag::new(&result2), &Tag::new(&result));
        site.observe_var("VAR:result2".into(), Tag::new(&result2));
    }

    site.update(&mut value_uf);

    return result
}

// returns the n-th fib number (0-indexed) and 2^n
fn complex_func(iterations: u32) -> (u32, u32) {
    let mut value_uf = VALUE_UF.lock().unwrap();
    let mut site_ufs = SITE_UFS.lock().unwrap();
    let site = site_ufs.get_site(1);

    site.observe_var("VAR:iterations".into(), Tag::new(&iterations));
    value_uf.make_set(Tag::new(&iterations));
    
    let mut current: u32 = 0;
    site.observe_var("VAR:current".into(), Tag::new(&current));
    value_uf.make_set(Tag::new(&current));

    let mut next: u32 = 1;
    site.observe_var("VAR:next".into(), Tag::new(&next));
    value_uf.make_set(Tag::new(&next));

    let mut pows_of_two: u32 = 1;
    site.observe_var("VAR:pows_of_two".into(), Tag::new(&pows_of_two));
    value_uf.make_set(Tag::new(&pows_of_two));

    for i in 0..iterations {
        site.observe_var("VAR:i".into(), Tag::new(&i));
        value_uf.make_set(Tag::new(&i));
        value_uf.union(&Tag::new(&i), &Tag::new(&iterations));

        let tmp = next;
        site.observe_var("VAR:tmp".into(), Tag::new(&tmp));
        value_uf.make_set(Tag::new(&tmp));
        value_uf.union(&Tag::new(&tmp), &Tag::new(&next));

        value_uf.union(&Tag::new(&current), &Tag::new(&next));
        next = current + next;
        value_uf.make_set(Tag::new(&next));
        value_uf.union(&Tag::new(&next), &Tag::new(&current));

        current = tmp;
        value_uf.make_set(Tag::new(&current));
        value_uf.union(&Tag::new(&current), &Tag::new(&tmp));

        let old_tag = Tag::new(&pows_of_two);
        pows_of_two = pows_of_two + pows_of_two;
        value_uf.union(&old_tag, &Tag::new(&pows_of_two));

    }
    
    site.update(&mut value_uf);
    
    (current, pows_of_two)
}

fn tracked_helper(a: u32, b: u32) -> u32 {
    let mut value_uf = VALUE_UF.lock().unwrap();
    // value_uf.union()
    a + b
}

fn untracked_helper(a: u32, b: u32) -> u32 {
    a + b
}

fn nested_func() {
    let x = 10;
    let y = 20;

}
