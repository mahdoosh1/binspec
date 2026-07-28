#[macro_export]
macro_rules! array {
    ($value:expr ; $len:expr) => {{
        let len: usize = $len;
        (0..len)
            .map(|_| $value)
            .collect::<Result<Vec<_>, _>>()
    }};
    ($name:ty { $($params:expr),* } = $data:ident ; $len:expr) => {{
        array!(<$name>::read($data, ($($params),*)); $len)
    }};
    ($name:ty = $data:ident ; $len:expr) => {{
        array!($name {()} = $data; $len)
    }};
}

#[macro_export]
macro_rules! assert_spec {
    ($cond:expr) => {
        if !$cond {
            return $crate::spec_error!(
                "assertion failed: `{}`", stringify!($cond)
            );
        }
    };
    ($cond:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {
        if !$cond {
            return $crate::spec_error!(
                "assertion failed: `{}`: {}",
                stringify!($cond),
                format!($fmt $(, $arg)*)
            );
        }
    };
}

#[macro_export]
macro_rules! assert_spec_eq {
    ($left:expr, $right:expr) => {{
        let left_val = $left;
        let right_val = $right;
        if left_val != right_val {
            return $crate::spec_error!(
                "assertion failed: `{} == {}` (left: {:?}, right: {:?})",
                stringify!($left),
                stringify!($right),
                left_val,
                right_val
            );
        }
    }};
    ($left:expr, $right:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {{
        let left_val = $left;
        let right_val = $right;
        if left_val != right_val {
            return $crate::spec_error!(
                "assertion failed: `{} == {}` (left: {:?}, right: {:?}): {}",
                stringify!($left),
                stringify!($right),
                left_val,
                right_val,
                format!($fmt $(, $arg)*)
            );
        }
    }};
}

#[macro_export]
macro_rules! assert_spec_ne {
    ($left:expr, $right:expr) => {{
        let left_val = $left;
        let right_val = $right;
        if left_val == right_val {
            return $crate::spec_error!(
                "assertion failed: `{} != {}` (left: {:?}, right: {:?})",
                stringify!($left),
                stringify!($right),
                left_val,
                right_val
            );
        }
    }};
    ($left:expr, $right:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {{
        let left_val = $left;
        let right_val = $right;
        if left_val == right_val {
            return $crate::spec_error!(
                "assertion failed: `{} != {}` (left: {:?}, right: {:?}): {}",
                stringify!($left),
                stringify!($right),
                left_val,
                right_val,
                format!($fmt $(, $arg)*)
            );
        }
    }};
}

#[macro_export]
macro_rules! spec_error {
    ($fmt:expr $(, $arg:expr)* $(,)?) => {
        Err($crate::errors::SpecError::ValidationFailed(
            $crate::errors::ValidationError::fail(format!($fmt $(, $arg)*)),
        ))
    };
    ($offset:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {
        Err($crate::errors::SpecError::ValidationFailed(
            $crate::errors::ValidationError::fail_at(format!($fmt $(, $arg)*), $offset),
        ))
    };
}