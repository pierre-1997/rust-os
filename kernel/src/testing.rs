pub struct TestCase {
    pub name: &'static str,

    pub test: fn(),
}

const FILTER: Option<&'static str> = None; //Some("GateDescriptor");

pub fn test_runner(tests: &[&dyn Fn() -> TestCase]) {
    println!("Running {} tests", tests.len());

    for test in tests {
        let case = test();

        if let Some(filter) = FILTER {
            if !case.name.contains(filter) {
                continue;
            }
        }

        print!("{}", case.name);
        (case.test)();
        println!("[ok]");
    }
}

#[test_case]
fn trivial_assertion() -> TestCase {
    TestCase {
        name: "Trivial assertion... ",
        test: || assert_eq!(1, 1),
    }
}
