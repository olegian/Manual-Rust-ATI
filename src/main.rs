mod ati;
mod site;
mod tag;
mod union_find;

use ati::ATI;
use tag::Tag;

/*
 === Compiler Requirements  ===
 - Define what sites we want to analyze
   - Probably just all functions included in the source code, excluding
     all calls to functions which are defined in libraries, etc.
   - TODO: Is there a good way of getting a list of function identifiers?
 - In main:
   - Create new mutable ATI struct instance
   - Invoke ati.report() before any program exit point
 - For all structs:
   - Define a `IdentifierTag` struct, which recursively mirrors all fields,
     converting primatives to `var_tag: Tag` types, and all structs to the relevant
     `StructTag`.
 - For each tracked function (note this happens for `impl`s too):
   - Modify the signature to accept a Tag type variable for each parameter
     - TODO: Does this include all parameters? are there specific parameters that do not
       require instrumentation?
   - Modify the signature to accept a mutable reference to the ATI struct
   - Modify the return type to make return a tuple of (val, val_tag)
   - Invoke ati.get_site, passing in the function identifier
   - Invoke ati.update_site(site) before every return statement
   - For each statement in the function (anytime a value is being instrumented, if it is a struct
     then perform the instrumentation for all primative values in the data struct, using the 
     appropriate tags in the Tag struct):
     - If the statement is a `let` binding, which is receiving a value not from a function call,
       invoke ati.tracked, passing in the variable identifier, a reference to the value, and the
       site.
     - If the statement is a `let` binding, which is receiving a value from an instrumented function,
       invoke site.observe_var passing in the variable identifier, and a reference to the returned tag
     - If the statement is a `let` binding, which is receiving a value from a non-instrumented function,
       invoke ati.tracked, as above.
     - If the statement is a tracked function call:
       - For variable arguments (ones that have already been bound with a `let`), pass in the value
         followed by the tag that was created after the `let` statement by the ati.tracked call
       - For constant arguments, add a `let` statement above the function call, followed by an
         ati.untracked() call, to get a tag for the value, then pass in both to the function
     - If the statement is an untracked function call, invoke the function as written.
     - If the statement includes interacting variables (track a set of operations which define
       "interacting"), then add an ati.union_tags() call, passing a slice of all tags which
       have interacted.
*/

/*
    Generally: the instrumentation follows this pattern:
    1. Define new necessary Tag types:
    struct S { a: prim, b: struct }
        -->
    struct STag { a_tag: Tag, b: StructTag}

    2. Convert function signatures:
    fn ident(f1: prim, f2: prim, f3: struct) -> prim
        -->  
    fn ident(f1: prim, f1_tag: &Tag, f2: prim, f2_tag: &Tag, f3: struct, f3_tag: StructTag, ati: &mut ATI) -> (prim, Tag)

    3. process main:
    fn main() {
        1. create ATI instance
        2. create site
        3. for each binding let statement, add `tracked` / `untracked` calls:
        3.1) let a = 10
             let a_tag = ati.tracked(Ident(a), &a, &mut site)
        3.2) let (a, a_tag) = tracked_func();
             site.observe_var(Ident(a), &a_tag)
        3.3) let a = untracked_func();
             let a_tag = ati.tracked(Ident(a), &a, &mut site)
        4. for each function call:
        4.1) let a = tracked_func(f1, f2)  (f1, f2 are variables)
                -->
             let (a, a_tag) = tracked_func(f1, f1_tag, f2, f2_tag, &mut ati)
        4.2) let a = tracked_func(f1, 10)  (f2 is some inline constant)
                -->
            let f2 = 10
             let f2_tag = ati.untracked(&f2) 
             let a = tracked_func(f1, f1_tag, f2, f2_tag, &mut ati)
        4.3) let a = untracked_func(f1, f2)
                -->
            let a_tag = ati.tracked(Ident(a), &a, &mut site)
        5. For all interaction sites, add ati.union_tags(&[&tag1, &tag2, &tag3])
        6. Before each program exit:
        6.1) ati.update_site(site)
        6.2) ati.report()
    }

    4. process all user defined functions:
    fn user_function(f1: prim, f1_tag: &Tag, f2: struct, f2_tag: StructTag, ati: &mut ATI) -> (prim, Tag) {
        1. create site
        2. for each formal:
        2.1) if formal is a primative: site.observe_var(Ident(f1), f1_tag)
        2.2) if formal is a struct, for all fields `a`: site.observe_var(Ident(f2.a), f2_tag.a_tag)
        3. for each binding let statement, add `tracked` / `untracked` calls:
        3.1) let a = 10
             let a_tag = ati.tracked(Ident(a), &a, &mut site)
        3.2) let (a, a_tag) = tracked_func();
             site.observe_var(Ident(a), &a_tag)
        3.3) let a = untracked_func();
             let a_tag = ati.tracked(Ident(a), &a, &mut site)
        4. for each function call:
        4.1) let a = tracked_func(f1, f2)  (f1, f2 are variables)
                -->
             let (a, a_tag) = tracked_func(f1, f1_tag, f2, f2_tag, ati)  // NOTE: passing &mut ati here
        4.2) let a = tracked_func(f1, 10)  (f2 is some inline constant here that is never bound to a var)
                -->
            let f2 = 10
            let f2_tag = ati.untracked(&f2) 
            let a = tracked_func(f1, f1_tag, f2, f2_tag, &mut ati)
        5. For all interaction sites, add ati.union_tags(&[&tag1, &tag2, &tag3])
        6. Before each function return:
        6.1) ati.update_site(site)
        6.2) return (val, val_tag)
    }

*/

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
        to track it. instead we leave it untracked, which provides the required 
        tags, however it does not have the site observe the variable
    */
    let a3 = 30;
    let a3_tag = ati.untracked(&a3);

    let b3 = 300;
    let b3_tag = ati.untracked(&b3);

    doubled_func(a3, &a3_tag, b3, &b3_tag, &mut ati);

    let iterations = 5;
    let iterations_tag = ati.tracked(stringify!(iterations), &iterations, &mut site);
    complex_func(iterations, &iterations_tag, &mut ati);

    uses_structs(&mut ati);

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

struct Data {
    a: u32,
    b: String,
    c: Inner,
}

struct DataTag {
    a_tag: Tag,
    b_tag: Tag,
    c_tag: InnerTag,
}

struct Inner {
    a: u32,
}

struct InnerTag {
    a_tag: Tag,
}

impl Data {
    pub fn new(ati: &mut ATI) -> (Self, DataTag) {
        let mut site = ati.get_site(stringify!(Data::new));
        let a = 10;
        let a_tag = ati.tracked(stringify!(Data::a), &a, &mut site);

        let b = "hello".to_owned();
        let b_tag = ati.tracked(stringify!(Data::b), &b, &mut site);

        let inner_a = 20;
        let inner_a_tag = ati.tracked(stringify!(Inner::a), &inner_a, &mut site);

        let inner = Inner { a: inner_a };
        let inner_tag = InnerTag { a_tag: inner_a_tag };

        ati.update_site(site);

        (
            Data { a, b, c: inner },
            DataTag {
                a_tag,
                b_tag,
                c_tag: inner_tag,
            },
        )
    }
}

fn accepts_struct_add_fields(data: &mut Data, data_tag: &mut DataTag, ati: &mut ATI) {
    let mut site = ati.get_site(stringify!(accepts_struct_add_fields));

    site.observe_var(stringify!(data.a), &data_tag.a_tag);
    site.observe_var(stringify!(data.b), &data_tag.b_tag);
    site.observe_var(stringify!(data.c.a), &data_tag.c_tag.a_tag);

    data.c.a += data.a;
    ati.union_tags(&[&data_tag.a_tag, &data_tag.c_tag.a_tag]);

    ati.update_site(site)
}

fn uses_structs(ati: &mut ATI) {
    let mut site = ati.get_site(stringify!(uses_structs));

    let (mut d, mut d_tag) = Data::new(ati);
    site.observe_var(stringify!(d.a), &d_tag.a_tag);
    site.observe_var(stringify!(d.b), &d_tag.b_tag);
    site.observe_var(stringify!(d.c.a), &d_tag.c_tag.a_tag);


    accepts_struct_add_fields(&mut d, &mut d_tag, ati);

    ati.update_site(site);
}
