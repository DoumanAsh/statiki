type RingBuffer<T> = statiki::RingBuffer<T, 512>;

#[test]
fn test_ring_buffer() {
    let mut queue = RingBuffer::<usize>::new();
    assert_eq!(queue.capacity(), 512);
    assert_eq!(queue.size(), 0);
    assert!(queue.is_empty());

    for idx in 0..queue.capacity()+9 {
        queue.push(idx);
    }
    assert!(!queue.is_empty());
    assert_eq!(queue.size(), 512);

    for expected_item in 9..queue.capacity()+9 {
        let value = queue.pop().expect("Value");
        assert_eq!(value, expected_item);
    }

    assert!(queue.pop().is_none());
    assert!(queue.is_empty());
    assert_eq!(queue.size(), 0);

    for idx in 0..queue.capacity()+9 {
        queue.push(idx);
    }
    assert!(!queue.is_empty());

    queue.clear();
    assert!(queue.pop().is_none());
    assert!(queue.is_empty());
    assert_eq!(queue.size(), 0);

    for idx in 0..9 {
        queue.push(idx);
    }
    assert!(!queue.is_empty());
    assert_eq!(queue.size(), 9);
    for expected_item in 0..9 {
        let value = queue.pop().expect("Value");
        assert_eq!(value, expected_item);
    }

    for idx in 0..queue.capacity() {
        assert!(queue.try_push(idx).is_none());
    }
    assert!(!queue.is_empty());
    assert_eq!(queue.size(), 512);
    assert_eq!(queue.try_push(999), Some(999));
}
