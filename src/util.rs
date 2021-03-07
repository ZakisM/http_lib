pub fn slice_find_to_end(main: &[u8], end: &[u8]) -> Option<usize> {
    let len = end.len();

    if len == 0 {
        None
    } else {
        main.windows(len)
            .enumerate()
            .find(|(_, chunk)| *chunk == end)
            .map(|(i, _)| i)
    }
}

#[cfg(test)]
mod tests {
    use crate::util::slice_find_to_end;

    #[test]
    fn test_slice_find_to_end() {
        let slice: [u8; 10] = [72, 84, 84, 80, 47, 50, 13, 10, 13, 10];

        let res = slice_find_to_end(&slice, &[13, 10, 13, 10]).expect("Failed to find to_end");

        assert_eq!(&slice[..res], [72, 84, 84, 80, 47, 50]);
    }
}
