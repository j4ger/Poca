#[cfg(test)]
#[macro_use]
extern crate lazy_static;

mod tests {
    use std::sync::{Arc, Mutex};

    use serde::{Deserialize, Serialize};
    use unroll_server::{DataHandle, Unroll};

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
    struct TestStruct {
        test_field: String,
        test_bool: bool,
    }

    lazy_static! {
        static ref UNROLL: Unroll = Unroll::new("localhost:1120");
        static ref HANDLE1: DataHandle<i32> = UNROLL.data("test1", 1);
        static ref HANDLE2: DataHandle<String> = UNROLL.data("test2", "test2".to_string());
        static ref HANDLE3: DataHandle<TestStruct> = UNROLL.data(
            "test3",
            TestStruct {
                test_field: "test_field".to_string(),
                test_bool: true
            }
        );
        static ref HANDLE4: DataHandle<Vec<i32>> = UNROLL.data("test4", vec![1, 2, 3]);
    }

    #[test]
    fn setting_and_getting() {
        HANDLE1.set(1);
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

    #[test]
    fn on_change_handler() {
        let watcher = Arc::new(Mutex::new(false));
        let watcher_clone = watcher.clone();
        HANDLE1.on_change(move |new_value| {
            println!("{:?}", new_value);
            let mut writer = watcher_clone.lock().unwrap();
            *writer = true;
        });

        HANDLE1.set(3);
        assert_eq!(*(watcher.lock().unwrap()), true);
    }

    #[test]
    fn on_change_handler_with_inner_set() {
        let handle5 = UNROLL.data("test5", true);
        HANDLE4.on_change(move |_new_value| {
            handle5.set(false);
        });
        HANDLE4.set(vec![4, 5, 6]);
    }
}
