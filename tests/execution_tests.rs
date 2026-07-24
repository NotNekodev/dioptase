use std::{
    env,
    error::Error,
    path::PathBuf,
    process::{Command, Stdio},
};

const TEST_CP: &str = "./tests/test_cp";

const FIXTURES: [TestFixture; 20] = [
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
        source_path: "pop_test/PopTest.java",
        entry_class: "PopTest",
        class_path: "pop_test",
        expected_return: 7,
    },
    TestFixture {
        source_path: "array_basic_test/ArrayBasicTest.java",
        entry_class: "ArrayBasicTest",
        class_path: "array_basic_test",
        expected_return: 35,
    },
    TestFixture {
        source_path: "array_loop_test/ArrayLoopTest.java",
        entry_class: "ArrayLoopTest",
        class_path: "array_loop_test",
        expected_return: 90,
    },
    TestFixture {
        source_path: "object_array_test/ObjectArrayTest.java",
        entry_class: "ObjectArrayTest",
        class_path: "object_array_test",
        expected_return: 2,
    },
    TestFixture {
        source_path: "invoke_virtual_test/InvokeVirtualTest.java",
        entry_class: "InvokeVirtualTest",
        class_path: "invoke_virtual_test",
        expected_return: 2,
    },
    TestFixture {
        source_path: "ldc_test/LdcTest.java",
        entry_class: "LdcTest",
        class_path: "ldc_test",
        expected_return: 106,
    },
    TestFixture {
        source_path: "getstatic_test/GetStaticTest.java",
        entry_class: "GetStaticTest",
        class_path: "getstatic_test",
        expected_return: 42,
    },
    TestFixture {
        source_path: "putstatic_test/PutStaticTest.java",
        entry_class: "PutStaticTest",
        class_path: "putstatic_test",
        expected_return: 30,
    },
    TestFixture {
        source_path: "clinit_test/ClinitTest.java",
        entry_class: "ClinitTest",
        class_path: "clinit_test",
        expected_return: 42,
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
        let _ = Command::new("echo")
            .arg(format!(
                "Test {} ({}) expected value: {}",
                self.entry_class, self.source_path, self.expected_return
            ))
            .stdout(Stdio::inherit())
            .output()?;

        let compile_output = Command::new("javac")
            .arg("-source")
            .arg("8")
            .arg("-target")
            .arg("8")
            .arg("-bootclasspath")
            .arg(TEST_CP)
            .arg(format!("./tests/fixtures/{}", &self.source_path))
            .output()?;

        if !compile_output.status.success() {
            return Err(format!(
                "javac failed for {}:\n{}",
                &self.source_path,
                String::from_utf8_lossy(&compile_output.stderr)
            )
            .into());
        }

        let combined_cp = env::join_paths([
            PathBuf::from(format!("./tests/fixtures/{}", &self.class_path)),
            PathBuf::from(TEST_CP),
        ])?;

        let run_output = Command::new("./target/debug/dioptase")
            .arg("--cp")
            .arg(&combined_cp)
            .arg(&self.entry_class)
            .arg("--no-ext")
            .status()?;

        let res = run_output.code().ok_or(format!(
            "Failed to get return code from {}",
            &self.entry_class
        ))?;

        let _ = Command::new("echo")
            .arg(format!("\t-> Test {} returned {}", self.entry_class, res))
            .stdout(Stdio::inherit())
            .output()?;

        assert_eq!(
            res, self.expected_return,
            "{} returned {} but expected {}",
            self.entry_class, res, self.expected_return
        );

        Ok(())
    }
}

#[test]
fn execution_tests() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    verify_javac()?;
    compile_test_cp()?;
    for fixture in FIXTURES {
        fixture.test()?;
    }
    Ok(())
}

fn verify_javac() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    Command::new("javac").output()?;
    Ok(())
}

fn compile_test_cp() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let object_src = format!("{}/java/lang/Object.java", TEST_CP);

    let object_output = Command::new("javac")
        .arg("-source")
        .arg("8")
        .arg("-target")
        .arg("8")
        .arg("-bootclasspath")
        .arg(TEST_CP)
        .arg("-d")
        .arg(TEST_CP)
        .arg(&object_src)
        .output()?;

    if !object_output.status.success() {
        return Err(format!(
            "javac failed compiling test_cp Object.java:\n{}",
            String::from_utf8_lossy(&object_output.stderr)
        )
        .into());
    }

    let remaining_sources = [
        format!("{}/java/lang/String.java", TEST_CP),
        format!("{}/java/lang/System.java", TEST_CP),
    ];

    let remaining_output = Command::new("javac")
        .arg("-source")
        .arg("8")
        .arg("-target")
        .arg("8")
        .arg("-bootclasspath")
        .arg(TEST_CP)
        .arg("-d")
        .arg(TEST_CP)
        .args(&remaining_sources)
        .output()?;

    if !remaining_output.status.success() {
        return Err(format!(
            "javac failed compiling test_cp java.lang stubs:\n{}",
            String::from_utf8_lossy(&remaining_output.stderr)
        )
        .into());
    }

    Ok(())
}
