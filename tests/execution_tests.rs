use std::{error::Error, process::Command};

const FIXTURES: [TestFixture; 12] = [
    TestFixture {
        source_path: "add_test/AddTest.java",
        entry_class: "AddTest",
        class_path: "add_test",
        expected_return: 8,
    },
    TestFixture {
        source_path: "factorial/Factorial.java",
        entry_class: "Factorial",
        class_path: "factorial",
        expected_return: 120,
    },
    TestFixture {
        source_path: "optimized_add_test/OptimizedAddTest.java",
        entry_class: "OptimizedAddTest",
        class_path: "optimized_add_test",
        expected_return: 75,
    },
    TestFixture {
        source_path: "return_test/ReturnTest.java",
        entry_class: "ReturnTest",
        class_path: "return_test",
        expected_return: 69,
    },
    TestFixture {
        source_path: "heap_test/HeapTest.java",
        entry_class: "HeapTest",
        class_path: "heap_test",
        expected_return: 42,
    },
    TestFixture {
        source_path: "goto_test/GotoTest.java",
        entry_class: "GotoTest",
        class_path: "goto_test",
        expected_return: 10,
    },
    TestFixture {
        source_path: "ifeq_test/IfEqTest.java",
        entry_class: "IfEqTest",
        class_path: "ifeq_test",
        expected_return: 42,
    },
    TestFixture {
        source_path: "iflt_ifge_test/IfLtIfGeTest.java",
        entry_class: "IfLtIfGeTest",
        class_path: "iflt_ifge_test",
        expected_return: 3,
    },
    TestFixture {
        source_path: "ifne_test/IfNeTest.java",
        entry_class: "IfNeTest",
        class_path: "ifne_test",
        expected_return: 100,
    },
    TestFixture {
        source_path: "ifnull_test/IfNullTest.java",
        entry_class: "IfNullTest",
        class_path: "ifnull_test",
        expected_return: 1,
    },
    TestFixture {
        source_path: "ifnonnull_test/IfNonNullTest.java",
        entry_class: "IfNonNullTest",
        class_path: "ifnonnull_test",
        expected_return: 55,
    },
    TestFixture {
        source_path: "goto_test/GotoTest.java",
        entry_class: "GotoTest",
        class_path: "goto_test",
        expected_return: 10,
    },
];

struct TestFixture<'a> {
    /// Path to source file
    ///
    /// I would have made this a &Path but idk how to construct at compile time :c
    source_path: &'a str,
    entry_class: &'a str,
    class_path: &'a str,
    expected_return: i32,
}

impl<'a> TestFixture<'a> {
    pub fn test(&self) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        Command::new("javac")
            .arg("-source")
            .arg("8")
            .arg("-target")
            .arg("8")
            .arg(format!("./tests/fixtures/{}", &self.source_path))
            .output()?;
        let res = Command::new("./target/debug/dioptase")
            .arg("--cp")
            .arg(format!("./tests/fixtures/{}", &self.class_path))
            .arg(&self.entry_class)
            .status()?
            .code()
            .ok_or(format!(
                "Failed to get return code from {}",
                &self.entry_class
            ))?;

        assert_eq!(res, self.expected_return);
        Ok(())
    }
}

#[test]
fn execution_tests() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    verify_javac()?;

    for fixture in FIXTURES {
        fixture.test()?;
    }

    Ok(())
}

fn verify_javac() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    Command::new("javac").output()?;
    Ok(())
}
