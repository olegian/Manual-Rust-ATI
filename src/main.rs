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
static VALUE_UF: LazyLock<Mutex<UnionFind<String>>> = LazyLock::new(|| Mutex::new(UnionFind::new()));

/// Tracks (potentially multiple) program sites which are under analysis.
/// At the beginning of a site, create a new Site using `site_ufs.get_site(id)`.
/// At the end of a site, call `site_ufs.get_site(id).update(value_uf)`.
/// View `site.rs` for more information.
static SITE_UFS: LazyLock<Mutex<Sites>> = LazyLock::new(|| Mutex::new(Sites::new()));

fn main() {
    simple_func(10, 100);
    simple_func(20, 200);

    // without this line, we should see two abstract type sets in the output
    // due to the conditional on line 118, otherwise all variables will be in the same sets
    // simple_func(30, 300); 

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

    value_uf.make_set(format!("VAL:{}", x));  // for parameter input
    value_uf.make_set(format!("VAL:{}", y));
    site.observe_var("VAR:x".into(), format!("VAL:{}", x));
    site.observe_var("VAR:y".into(), format!("VAL:{}", y));

    let a: u32 = 2;
    value_uf.make_set(format!("LIT:{}", 2));
    value_uf.make_set(format!("VAL:{}", a));
    value_uf.union(&format!("LIT:{}", 2), &format!("VAL:{}", a));
    site.observe_var("VAR:a".into(), format!("VAL:{}", a));

    let b: u32 = 3;
    value_uf.make_set(format!("LIT:{}", 3));
    value_uf.make_set(format!("VAL:{}", b));
    value_uf.union(&format!("LIT:{}", 3), &format!("VAL:{}", b));
    site.observe_var("VAR:b".into(), format!("VAL:{}", b));

    let result = a + x;
    value_uf.make_set(format!("VAL:{}", result));
    value_uf.union(&format!("VAL:{}", result), &format!("VAL:{}", a));
    value_uf.union(&format!("VAL:{}", result), &format!("VAL:{}", x));
    site.observe_var("VAR:result".into(), format!("VAL:{}", result));

    let test = b + y;
    value_uf.make_set(format!("VAL:{}", test));
    value_uf.union(&format!("VAL:{}", test), &format!("VAL:{}", b));
    value_uf.union(&format!("VAL:{}", test), &format!("VAL:{}", y));
    site.observe_var("VAR:test".into(), format!("VAL:{}", test));

    if test > 300 {
        // TODO: THIS IMPLEMENTAITON REQUIRES SSA FORM!
        // is that fine ??? a smarter choice of tag probably addresses this.
        // basically because value_uf has to incorporate both old result here and result2 with tags for proper merging
        let result2 = result + test;
        value_uf.make_set(format!("VAL:{}", result2));
        value_uf.union(&format!("VAL:{}", result2), &format!("VAL:{}", result));
        value_uf.union(&format!("VAL:{}", result2), &format!("VAL:{}", test));
        site.observe_var("VAR:result2".into(), format!("VAL:{}", result2));
    }

    site.update(&mut value_uf);

    return result
}
