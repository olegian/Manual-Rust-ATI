mod ati;
mod site;
mod tag;
mod union_find;

use ati::ATI;
use tag::Tag;

fn main() {
    let mut ati = ATI::new();
    let mut site = ati.get_site(stringify!(main));

    /*
        In instrumenting the following call:
            a = 10
            b = 100
            doubled_func(a, b)

        we bind 10 and 100 to "a" and "b", so we use a tracked
        call to register the tag with this site
    */
    let a1 = 10;
    let a1_tag = ati.tracked(stringify!(a1), &a1, &mut site);

    let b1 = 100;
    let b1_tag = ati.tracked(stringify!(b1), &b1, &mut site);

    doubled_func(a1, &a1_tag, b1, &b1_tag, &mut ati);

    let a2 = 20;
    let a2_tag = ati.tracked(stringify!(a2), &a2, &mut site);

    let b2 = 200;
    let b2_tag = ati.tracked(stringify!(b2), &b2, &mut site);

    doubled_func(a2, &a2_tag, b2, &b2_tag, &mut ati);

    // without these lines, we should see two abstract type sets in the output
    /*
        In instrumenting the following call:
            doubled_func(30, 300)

        we do not bind 30 or 300 to any variable at this site, so we do not want
        to track it. instead we leave it untracked
    */
    let a3 = 30;
    let a3_tag = ati.untracked(&a3);

    let b3 = 300;
    let b3_tag = ati.untracked(&b3);

    doubled_func(a3, &a3_tag, b3, &b3_tag, &mut ati);

    let iterations = 5;
    let iterations_tag = ati.tracked(stringify!(iterations), &iterations, &mut site);
    complex_func(iterations, &iterations_tag, &mut ati);

    ati.update_site(site);
    ati.report()
}

/// This is an example of a function that we want to analyze
/// Each white space seperated line is a line of code we are analyzing,
/// the first of which is the actual code, the following are the added
/// lines to perform ATI.
fn doubled_func(x: u32, x_tag: &Tag, y: u32, y_tag: &Tag, ati: &mut ATI) {
    let mut site = ati.get_site(stringify!(doubled_func));
    site.observe_var(stringify!(x), x_tag);
    site.observe_var(stringify!(y), y_tag);

    let a: u32 = 2;
    let a_tag: Tag = ati.tracked(stringify!(a), &a, &mut site);

    let b: u32 = 2;
    let b_tag: Tag = ati.tracked(stringify!(b), &b, &mut site);

    let result: u32 = a + x;
    let result_tag: Tag = ati.tracked(stringify!(result), &result, &mut site);
    ati.union_tags(&[&a_tag, &x_tag, &result_tag]);

    let test: u32 = b + y;
    let test_tag: Tag = ati.tracked(stringify!(test), &test, &mut site);
    ati.union_tags(&[&b_tag, &y_tag, &test_tag]);

    if test > 300 {
        /*
            This is adding these two values without any cross-function boundaries.
        */
        // let merged: u32 = result + test;
        // let merged_tag: Tag = ati.tracked(stringify!(merged), &merged, &mut site);
        // ati.union_tags(&[&merged_tag, &result_tag, &test_tag]);

        /*
            untracked_add() is a function that we do not instrument,
            like a library function call which our compiler will NOT add
            any tracking code to.

            Because it is untracked, the function call will not add the return
            value to the value_uf, and therefore we treat the value as being
            "created" in this scope, and therefore tracked.
        */
        let merged = untracked_add(result, test);
        let merged_tag: Tag = ati.tracked(stringify!(merged), &merged, &mut site);

        /*
            tracked_add() is a function we do instrument.
            tracked_add is going to be where the return value is created and added
            into the value_uf, so therefore, we do not "create" a new value in this scope
            we just have to observe that we have a new variable that binded the return value
        */
        // let (merged, merged_tag) = tracked_add(result, &result_tag, test, &test_tag, ati);
        // site.observe_var(stringify!(merged), &merged_tag);
    }

    ati.update_site(site);
}

// returns the n-th fib number (0-indexed) and 2^n
fn complex_func(iterations: u32, iterations_tag: &Tag, ati: &mut ATI) -> (u32, u32) {
    let mut site = ati.get_site(stringify!(complex_func));
    site.observe_var(stringify!(iterations), iterations_tag);

    let mut current: u32 = 0;
    let current_tag = ati.tracked(stringify!(current), &current, &mut site);

    let mut next: u32 = 1;
    let next_tag = ati.tracked(stringify!(next), &next, &mut site);

    let mut pows_of_two: u32 = 1;
    let pows_of_two_tag = ati.tracked(stringify!(pows_of_two), &pows_of_two, &mut site);

    for i in 0..iterations {
        let i_tag = ati.tracked(stringify!(i), &i, &mut site);
        ati.union_tags(&[&i_tag, &iterations_tag]);

        let tmp = next;
        let tmp_tag = ati.tracked(stringify!(tmp), &tmp, &mut site);
        ati.union_tags(&[&tmp_tag, &next_tag]);

        // TODO: with SSA, this problem goes away, where an old tag has to be merged before the statement
        ati.union_tags(&[&current_tag, &next_tag]);
        next = current + next;
        let next_tag = ati.tracked(stringify!(next), &next, &mut site);
        ati.union_tags(&[&next_tag, &current_tag]);

        current = tmp;
        let current_tag = ati.tracked(stringify!(current), &current, &mut site);
        ati.union_tags(&[&current_tag, &tmp_tag]);

        // TODO: same sort of thing here, awkward tag management due to no SSA
        let old_tag = pows_of_two_tag.clone();
        pows_of_two = pows_of_two + pows_of_two;
        let pows_of_two_tag = ati.tracked(stringify!(pows_of_two), &pows_of_two, &mut site);
        ati.union_tags(&[&pows_of_two_tag, &old_tag])
    }

    ati.update_site(site);

    (current, pows_of_two)
}

/// a "library function", which is not instrumented for ATI
fn untracked_add(a: u32, b: u32) -> u32 {
    let res = a + b;
    res
}

/// a function instrumented for ATI
fn tracked_add(a: u32, a_tag: &Tag, b: u32, b_tag: &Tag, ati: &mut ATI) -> (u32, Tag) {
    let mut site = ati.get_site(stringify!(tracked_add));
    site.observe_var(stringify!(a), a_tag);
    site.observe_var(stringify!(b), b_tag);

    let res = a + b;
    let res_tag = ati.tracked(stringify!(res), &res, &mut site);
    ati.union_tags(&[&a_tag, &b_tag, &res_tag]);

    ati.update_site(site);

    // NOTE: all cross-function boundary values need to also pass tags
    // which includes not just the parameters, but returns as well.
    (res, res_tag)
}

struct Inner {
    a: u32,
    a_tag: Tag,
}

struct Data {
    a: u32,
    a_tag: Tag,

    b: String,
    b_tag: Tag,

    c: Inner,
}

impl Data {
    pub fn new(ati: &mut ATI) -> Self {
        let mut site = ati.get_site(stringify!(Data::new));
        let a = 10;
        let a_tag = ati.tracked(stringify!(Data::a), &a, &mut site);

        let b = "hello".to_owned();
        let b_tag = ati.tracked(stringify!(Data::b), &b, &mut site);

        let inner_a = 20;
        let inner_a_tag = ati.tracked(stringify!(Inner::a), &inner_a, &mut site);

        let inner = Inner {
            a: inner_a,
            a_tag: inner_a_tag,
        };

        Data {
            a,
            a_tag,
            b,
            b_tag,
            c: inner,
        }
    }
}

fn accepts_struct(data: Data, ati: &mut ATI) {
    let mut site = ati.get_site(stringify!(accepts_struct));
}
