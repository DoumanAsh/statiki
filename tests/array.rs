use core::fmt::Write;
use core::sync::atomic::{AtomicUsize, Ordering};

use statiki::Array;

#[test]
fn test_array_iterator() {
    let mut array = Array::<usize, 512>::new();

    for idx in 1..=500 {
        assert!(array.push(idx).is_none());
    }

    for (idx, elem) in array.into_iter().enumerate() {
        assert_eq!(idx + 1, elem);
    }
}

#[test]
fn test_array_destructor() {
    static COUNT: AtomicUsize = AtomicUsize::new(0);
    #[derive(Default)]
    struct Lolka {
    }

    impl Drop for Lolka {
        fn drop(&mut self) {
            COUNT.fetch_add(1, Ordering::Relaxed);
        }
    }

    Array::<Lolka, 512>::new();

    assert_eq!(COUNT.load(Ordering::Relaxed), 0);

    let mut array = Array::<Lolka, 512>::new();
    array.resize_default(500);
    assert_eq!(array.len(), 500);
    array.truncate(400);
    assert_eq!(array.len(), 400);
    assert_eq!(COUNT.load(Ordering::Relaxed), 100);

    let mut idx = 0;
    for _elem in array {
        idx += 1;
    }
    assert_eq!(idx, 400);

    assert_eq!(COUNT.load(Ordering::Relaxed), 500);
}

#[cfg(feature = "std")]
#[test]
fn test_array_write() {
    use std::io::Write;

    const SIZE: usize = 100;
    let mut array = Array::<_, 512>::new();
    let data = [0u8; SIZE];

    let full_write_num = array.capacity() / SIZE;

    for idx in 0..full_write_num {
        let res = array.write(&data).expect("To successfully write");
        assert_eq!(res, SIZE);
        assert_eq!(array.len(), (idx+1) * SIZE);
    }

    let res = array.write(&data).expect("To successfully write");
    assert_eq!(res, array.capacity() - full_write_num * SIZE);
    assert_eq!(array.len(), array.capacity());

    let res = array.write(&data).expect("To successfully write");
    assert_eq!(res, 0);
}

#[test]
fn test_array_clone() {
    let mut array = Array::<_, 512>::new();
    for idx in 0..array.capacity() {
        assert!(array.push(idx).is_none());
    }

    let mut cloned = array.clone();
    assert_eq!(cloned.len(), array.len());
    assert_eq!(cloned, array);
    assert_eq!(cloned, array.as_slice());

    for idx in (0..array.capacity()).rev() {
        assert_eq!(idx, cloned.pop().unwrap());
    }
}

#[test]
fn test_array() {
    let mut array = Array::<_, 512>::new();
    assert_eq!(array.capacity(), 512);
    assert!(array.is_empty());
    assert_eq!(format!("{:?}", &array), "[]");

    assert!(array.push(15).is_none());
    assert!(!array.is_empty());

    let elem = array.pop().expect("To get value");
    assert_eq!(15, elem);
    assert!(array.pop().is_none());

    for idx in 0..array.capacity() {
        assert!(array.push(idx).is_none());
    }

    assert!(array.push(500).is_some());

    for idx in (0..array.capacity()).rev() {
        let elem = array.pop().expect("To get value");
        assert_eq!(idx, elem);
    }

    assert!(array.pop().is_none());
    array.clear();

    let mut expected_format = String::new();
    let _ = write!(&mut expected_format, "[");
    for idx in 0..array.capacity() {
        assert!(array.push(idx).is_none());
        let _ = write!(&mut expected_format, "{}, ", idx);
    }
    expected_format.pop();
    expected_format.pop();
    let _ = write!(&mut expected_format, "]");

    assert_eq!(format!("{:?}", &array), expected_format);

    array.resize(array.capacity() / 2, 1);
    assert_eq!(array.len(), array.capacity() / 2);

    array.clear();
    assert!(array.is_empty());
    assert!(array.pop().is_none());

    array.resize(array.capacity(), 1);
    assert_eq!(array.len(), array.capacity());

    for _ in 0..array.capacity() {
        let item = array.pop().expect("To get value");
        assert_eq!(item, 1);
    }

    assert!(array.is_empty());
    assert!(array.pop().is_none());

    array.resize_default(array.capacity() / 2);
    assert_eq!(array.len(), array.capacity() / 2);

    for _ in 0..array.capacity() / 2 {
        let item = array.pop().expect("To get value");
        assert_eq!(item, 0);
    }

    assert!(array.is_empty());
    assert!(array.pop().is_none());
}

