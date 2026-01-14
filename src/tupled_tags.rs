use crate::{
    ati::{ATI, ATI_ANALYSIS},
    tag::TaggedValue,
};

pub fn tupled_main() {
    // we want a to become a TaggedValue, but it cannot until a is assigned an address!
    // address doesnt until codegen -- is there a different value that can be used for IDing?
    // just use a counter during tag initialization?
    let mut site = ATI_ANALYSIS.lock().unwrap().get_site(stringify!(tupled_main));

    // a = 10;
    let a = site.bind(stringify!(a), ATI::track(10));

    // b = 20;
    let b = site.bind(stringify!(b), ATI::track(20));

    // c = a + b
    let c = site.bind(stringify!(c), a + b);

    // d = 100 + 200
    let d = site.bind(stringify!(d), ATI::track(100) + ATI::track(200));

    foo(c, d);
    uses_structs();

    // in main, aquiring this lock twice is almost def unnecessary
    ATI_ANALYSIS.lock().unwrap().update_site(site);
    ATI_ANALYSIS.lock().unwrap().report();
}

fn foo(x: TaggedValue<u32>, y: TaggedValue<u32>) {
    // prelude to register parameters as a part of the site
    let mut site = ATI_ANALYSIS.lock().unwrap().get_site(stringify!(foo));
    site.bind_param(stringify!(x), &x);
    site.bind_param(stringify!(y), &y);

    // constant = 1
    let constant = site.bind(
        stringify!(constant),
        ATI::track(100),
    );

    // x + constant > 500
    if x + constant > ATI::track(500) {
        // a1 = x + y
        let a1 = site.bind(stringify!(a1), x + y);
    }

    ATI_ANALYSIS.lock().unwrap().update_site(site);
}

struct Inner {
    a: TaggedValue<u32>,
}

struct MyStruct {
    a: TaggedValue<u32>,
    b: TaggedValue<u32>,
    c: Inner,
}

fn uses_structs() {
    let mut site = ATI_ANALYSIS.lock().unwrap().get_site(stringify!(uses_structs));

    // QUESTION: what should `tmp` be labeled as in the context of ATI?
    // Is anything even reported about it? Is it "untracked"?
    let tmp = Inner {
        a: ATI::track(10),
    };

    let my_struct = MyStruct {
        a: ATI::track(1),
        b: ATI::track(2),
        c: tmp,
    };

    // A = 100
    let A = site.bind(stringify!(A), ATI::track(100));

    // B = 200
    let B = site.bind(stringify!(B), ATI::track(200));

    // These lines test if the unification works via things within structs. A and B should have the same AT
    // var1 = my_struct.c.a + A
    // var2 = my_struct.c.a + B
    let var1 = site.bind(stringify!(var1), my_struct.c.a + A);
    let var2 = site.bind(stringify!(var2), my_struct.c.a + B);

    ATI_ANALYSIS.lock().unwrap().update_site(site);
}
