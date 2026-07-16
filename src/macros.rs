#[macro_export]
macro_rules! array {
    ($name:ty { $($params:expr),* } = $data:ident ; $len:expr) => {{
        let len: usize = $len;
        (0..len)
            .map(|_| <$name>::read($data, ($($params),*)))
            .collect::<Result<Vec<$name>, _>>()
    }};
    ($name:ty = $data:ident ; $len:expr) => {{
        let len: usize = $len;
        (0..len)
            .map(|_| <$name>::read($data, ()))
            .collect::<Result<Vec<$name>, _>>()
    }};
    ($value:expr ; $len:expr) => {{
        let len: usize = $len;
        (0..len)
            .map(|_| $value)
            .collect::<Result<Vec<_>, _>>()
    }};
}

#[macro_export]
macro_rules! assert_spec {
    ($cond:expr) => {
        if !$cond {
            $crate::bail_validation!(
                "assertion failed: `{}`", stringify!($cond)
            );
        }
    };
    ($cond:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {
        if !$cond {
            $crate::bail_validation!(
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
            $crate::bail_validation!(
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
            $crate::bail_validation!(
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
            $crate::bail_validation(
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
            $crate::bail_validation(
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
macro_rules! bail_validation {
    ($fmt:expr $(, $arg:expr)* $(,)?) => {
        return Err($crate::specs::SpecError::ValidationFailed(
            $crate::errors::ValidationError::fail(format!($fmt $(, $arg)*)),
        ));
    };
}