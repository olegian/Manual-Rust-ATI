mod union_find;
mod site;
mod tag;

use union_find::UnionFind;
use site::{Sites};
use std::{sync::{LazyLock, Mutex}};
use tag::Tag;

/// Global UF tracking value interaction sets.
/// Whenever two values are observed interacting, union together the tags (SetIds)
/// stored in this structure.
/// Any time a new value is used (either literal values, or a new value is bound to
/// a new variable with `let`), make a new set in this structure with a unique SetId
/// to represent the value.
static VALUE_UF: LazyLock<Mutex<UnionFind>> = LazyLock::new(|| Mutex::new(UnionFind::new()));

/// Tracks (potentially multiple) program sites which are under analysis.
/// At the beginning of a site, create a new Site using `site_ufs.get_site(id)`.
/// At the end of a site, call `site_ufs.get_site(id).update(value_uf)`.
/// View `site.rs` for more information.
static SITE_UFS: LazyLock<Mutex<Sites>> = LazyLock::new(|| Mutex::new(Sites::new()));

fn main() {
    // simple_func(10, 100);
    // simple_func(20, 200);

    // without this line, we should see two abstract type sets in the output
    // due to the conditional on line 118, otherwise all variables will be in the same sets
    // simple_func(30, 300); 

    // let res = complex_func(5);
    // println!("{:?}", res);

    nested_func();

    let site_ufs = SITE_UFS.lock().unwrap();
    site_ufs.print_analysis();
}

/// This is an example of a function that we want to analyze
/// Each white space seperated line is a line of code we are analyzing,
/// the first of which is the actual code, the following are the added
/// lines to perform ATI.
fn simple_func(x: u32, y: u32) -> u32 {
    let mut value_uf = VALUE_UF.lock().unwrap();
    let mut site_ufs = SITE_UFS.lock().unwrap();
    let site = site_ufs.get_site(0);  // create a new analysis site. View site.rs for more info

    value_uf.make_set(&x);  // for parameter input
    value_uf.make_set(&y);  // for parameter input
    site.observe_var(stringify!(x), &x);
    site.observe_var(stringify!(y), &y);

    let a: u32 = 2;
    value_uf.make_set(&a);
    site.observe_var(stringify!(a), &a);

    let b: u32 = 2;
    value_uf.make_set(&b);
    site.observe_var(stringify!(b), &b);

    value_uf.union_vals(&a, &x);
    let result = a + x;
    value_uf.make_set(&result);
    value_uf.union_vals(&result, &a);
    site.observe_var(stringify!(result), &result);

    value_uf.union_vals(&b, &y);
    let test = b + y;
    value_uf.make_set(&test);
    value_uf.union_vals(&test, &b);
    site.observe_var(stringify!(test), &test);

    if test > 300 {
        // TODO: THIS IMPLEMENTAITON REQUIRES SSA FORM!
        // is that fine ??? a smarter choice of tag probably addresses this.
        // problem comes from needing to register an interaction between
        // the value here in result, and the value stored in result2,
        // in otherwords, we cannot get rid of the old value, as we need to retain
        // it's tag for proper merging
        value_uf.union_vals(&result, &test);
        let result2 = result + test;
        value_uf.make_set(&result2);
        value_uf.union_vals(&result2, &result);
        site.observe_var(stringify!(result2), &result2);
    }

    site.update(&mut value_uf);

    return result
}

// returns the n-th fib number (0-indexed) and 2^n
fn complex_func(iterations: u32) -> (u32, u32) {
    let mut value_uf = VALUE_UF.lock().unwrap();
    let mut site_ufs = SITE_UFS.lock().unwrap();
    let site = site_ufs.get_site(1);

    site.observe_var(stringify!(iterations), &iterations);
    value_uf.make_set(&iterations);
    
    let mut current: u32 = 0;
    site.observe_var(stringify!(current), &current);
    value_uf.make_set(&current);

    let mut next: u32 = 1;
    site.observe_var(stringify!(next), &next);
    value_uf.make_set(&next);

    let mut pows_of_two: u32 = 1;
    site.observe_var(stringify!(pows_of_two), &pows_of_two);
    value_uf.make_set(&pows_of_two);

    for i in 0..iterations {
        site.observe_var(stringify!(i), &i);
        value_uf.make_set(&i);
        // TODO: try removing value form Tag struct
        // should a "new" i value be union_valsed with an "old" i value
        value_uf.union_vals(&i, &iterations);

        let tmp = next;
        site.observe_var(stringify!(tmp), &tmp);
        value_uf.make_set(&tmp);
        value_uf.union_vals(&tmp, &next);

        value_uf.union_vals(&current, &next);
        next = current + next;
        value_uf.make_set(&next);
        value_uf.union_vals(&next, &current);

        current = tmp;
        value_uf.make_set(&current);
        value_uf.union_vals(&current, &tmp);

        pows_of_two = pows_of_two + pows_of_two;
    }
    
    site.update(&mut value_uf);
    
    (current, pows_of_two)
}

// TODO: copy-by-value into this function means that the union here
// will not represent the interaction between the passed in values?
// does this mean that Tags *have* to be passed into the function as well?
fn tracked_helper(a: u32, b: u32) -> u32 {
    let mut value_uf = VALUE_UF.lock().unwrap();
    value_uf.union_vals(&a, &b);
    a + b
}

fn untracked_helper(a: u32, b: u32) -> u32 {
    a + b
}

fn nested_func() {
    let mut value_uf = VALUE_UF.lock().unwrap();
    let mut site_ufs = SITE_UFS.lock().unwrap();
    let site = site_ufs.get_site(1);

    let x = 10;
    value_uf.make_set(&x);
    site.observe_var(stringify!(x), &x);

    let y = 20;
    value_uf.make_set(&y);
    site.observe_var(stringify!(y), &y);

    // TODO: this is a problem, using global state requires obtaining a lock before you use 
    // value_uf, which means that the tracked_helper lock obtain stalls without this ugly workaround
    drop(value_uf);

    // should update x and y
    let res = tracked_helper(x, y);
    value_uf = VALUE_UF.lock().unwrap();
    value_uf.make_set(&res);
    site.observe_var(stringify!(res), &res);

    println!("D");

    site.update(&mut value_uf);
    println!("E");
}
