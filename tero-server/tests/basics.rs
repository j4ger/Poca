#[cfg(test)]
#[macro_use]
extern crate lazy_static;

mod tests {
    use serde::{Deserialize, Serialize};
    use tero_server::{DataHandle, Tero};

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
    struct TestStruct {
        test_field: String,
        test_bool: bool,
    }

    lazy_static! {
        static ref TERO: Tero = Tero::new("localhost:1120");
        static ref HANDLE1: DataHandle<i32> = TERO.data("test1", 1);
        static ref HANDLE2: DataHandle<String> = TERO.data("test2", "test2".to_string());
        static ref HANDLE3: DataHandle<TestStruct> = TERO.data(
            "test3",
            TestStruct {
                test_field: "test_field".to_string(),
                test_bool: true
            }
        );
    }

    #[test]
    fn setting_and_getting() {
        assert_eq!(*HANDLE1.get(), 1);

        HANDLE1.set(2);
        assert_eq!(*HANDLE1.get(), 2);

        assert_eq!(*HANDLE2.get(), "test2".to_string());

        HANDLE2.set("test3".to_string());
        assert_eq!(*HANDLE2.get(), "test3".to_string());

        assert_eq!(
            *HANDLE3.get(),
            TestStruct {
                test_field: "test_field".to_string(),
                test_bool: true
            }
        );

        HANDLE3.set(TestStruct {
            test_field: "test_field2".to_string(),
            test_bool: false,
        });
        assert_eq!(
            *HANDLE3.get(),
            TestStruct {
                test_field: "test_field2".to_string(),
                test_bool: false
            }
        );
    }
}
