pub struct TestCase {
    pub name: &'static str,

    pub test: fn(),
}

pub fn test_runner(tests: &[&dyn Fn() -> TestCase]) {
    println!("Running {} tests", tests.len());

    for test in tests {
        let case = test();

        print!("{}", case.name);
        (case.test)();
        println!("[ok]");
    }
}

#[test_case]
fn trivial_assertion() -> TestCase {
    TestCase {
        name: "Trivial assertion... ",
        test: || assert_eq!(1, 0),
    }
}
