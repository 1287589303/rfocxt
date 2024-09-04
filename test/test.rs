type Aint = i32;
static mut A: Aint = 1;

struct S1<T> {
    a: i32,
}

struct S2 {
    b: i32,
}

static mut S1_BOOL: S1<bool> = S1 { a: 0 };
static mut S2_STRUCT: S2 = S2 { b: 0 };

union U {
    s1: S1<bool>,
    s2: S2,
}

enum E {
    E1,
    E2,
}

struct C1<T, T1> {
    a: T,
}

impl<T, T1> C1<T, T1> {
    type S1int = i32;

    fn new(a: T) -> Self {
        C1 { a }
    }

    fn ct<T2>(&self, _a: T2) {
        //
    }

    fn ctt(&self, _b: T) {
        //
    }
}

struct C3<T, T1> {
    a: T,
}

impl<T, T1> C3<T, T1> {
    fn new(a: T, _b: T1) -> Self {
        C3 { a }
    }
}

struct C2 {
    a: i32,
}

trait Trait1 {
    fn tr() -> i32 {
        1
    }
}

impl Trait1 for C1 {
    fn tr() -> i32 {
        2
    }
}

fn func1<T>(_a: T) {
    let _b: i32;
}

fn func2(mut a: i32) {
    a = 1;
}

fn main() {
    let u = U { s1: S1 { a: 0 } };

    let e = E::E1;
    let c1 = C1::new('a' as char);
    c1.ct(true);
    let _c2 = C2 { a: 0 };
    let s = String::from("");
    func1(s);
    func2(1);
}
