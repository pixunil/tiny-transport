/// Like `assert_eq!`, but uses `{:#?}` for printing the value on failure.
#[macro_export]
macro_rules! assert_eq_alternate {
    ($left:expr, $right:expr $(,)?) => {{
        match (&$left, &$right) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    panic!(
                        r#"assertion failed: `(left == right)`
  left: `{:#?}`,
 right: `{:#?}`"#,
                        &*left_val, &*right_val
                    )
                }
            }
        }
    }};
}

#[macro_export]
macro_rules! map {
    () => (
        std::collections::HashMap::new()
    );
    ($($key:expr => $value:expr),* $(,)?) => ({
        let mut map = std::collections::HashMap::new();
        $(
            map.insert($key.into(), $value);
        )*
        map
    });
}

#[macro_export]
macro_rules! time {
    (Duration; $( $(:)? $time:literal )*) => {
        Duration::seconds(time!($($time),*))
    };
    ($(:)? $minute:literal : $second:literal) => {
        time!(0, $minute, $second)
    };
    ($(:)? $hour:literal : $minute:literal : $second:literal) => {
        time!($hour, $minute, $second)
    };
    ($minute:literal, $second:literal) => {
        time!(0, $minute, $second)
    };
    ($hour:literal, $minute:literal, $second:literal) => {
        $hour * 3600 + $minute * 60 + $second
    };
}

#[macro_export]
macro_rules! times {
    (Duration; $( $( $(:)? $time:literal )* ),*) => {{
        use $crate::time;
        vec![
            $( Duration::seconds(time!($($time),*)) ),*
        ]
    }};
    (Duration; +$start:expr, []) => {{
        let _start = $start;
        Vec::new()
    }};
    (Duration; +$start:expr, [ $( $( $(:)? $time:literal )* ),* ]) => {{
        use $crate::time;
        vec![
            $( Duration::seconds($start + time!($($time),*)) ),*
        ]
    }};
    (Duration; $start:expr, [ $( $( $(:)? $time:literal )* ),* ]) => {{
        use $crate::time;
        vec![
            Duration::seconds($start),
            $( Duration::seconds(time!($($time),*)) ),*
        ]
    }};
    ($( $( $(:)? $time:literal )* ),*) => {{
        use $crate::time;
        vec![
            $( time!($($time),*) ),*
        ]
    }};
    ($start:expr, [ $( $( $(:)? $time:literal )* ),* ]) => {{
        use $crate::time;
        vec![
            $start,
            $( time!($($time),*) ),*
        ]
    }};
}
