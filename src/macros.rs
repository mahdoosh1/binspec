#[macro_export]
macro_rules! array {
    ($name:path = $view:expr ; $len:expr) => {{
        array!($name {()} = $view; $len)
    }};
    ($name:path { $($params:expr),* } = $view:expr ; $len:expr) => {{
        array!(<$name>::read_from_view($view, ($($params),*)); $len)
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
            return Err($crate::spec_error!(
                "assertion failed: `{}`", stringify!($cond)
            ));
        }
    };
    ($cond:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {
        if !$cond {
            return Err($crate::spec_error!(
                "assertion failed: `{}`: {}",
                stringify!($cond),
                format!($fmt $(, $arg)*)
            ));
        }
    };
}

#[macro_export]
macro_rules! assert_spec_eq {
    ($left:expr, $right:expr) => {{
        let left_val = $left;
        let right_val = $right;
        if left_val != right_val {
            return Err($crate::spec_error!(
                "assertion failed: `{} == {}` (left: {:?}, right: {:?})",
                stringify!($left),
                stringify!($right),
                left_val,
                right_val
            ));
        }
    }};
    ($left:expr, $right:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {{
        let left_val = $left;
        let right_val = $right;
        if left_val != right_val {
            return Err($crate::spec_error!(
                "assertion failed: `{} == {}` (left: {:?}, right: {:?}): {}",
                stringify!($left),
                stringify!($right),
                left_val,
                right_val,
                format!($fmt $(, $arg)*)
            ));
        }
    }};
}

#[macro_export]
macro_rules! assert_spec_ne {
    ($left:expr, $right:expr) => {{
        let left_val = $left;
        let right_val = $right;
        if left_val == right_val {
            return Err($crate::spec_error!(
                "assertion failed: `{} != {}` (left: {:?}, right: {:?})",
                stringify!($left),
                stringify!($right),
                left_val,
                right_val
            ));
        }
    }};
    ($left:expr, $right:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {{
        let left_val = $left;
        let right_val = $right;
        if left_val == right_val {
            return Err($crate::spec_error!(
                "assertion failed: `{} != {}` (left: {:?}, right: {:?}): {}",
                stringify!($left),
                stringify!($right),
                left_val,
                right_val,
                format!($fmt $(, $arg)*)
            ));
        }
    }};
}

#[macro_export]
macro_rules! spec_error {
    ($fmt:expr $(, $arg:expr)* $(,)?) => {
        $crate::errors::SpecError::ValidationFailed(
            $crate::errors::ValidationError::fail(format!($fmt $(, $arg)*)),
        )
    };
    ($offset:expr; $fmt:expr $(, $arg:expr)* $(,)?) => {
        $crate::errors::SpecError::ValidationFailed(
            $crate::errors::ValidationError::fail_at(format!($fmt $(, $arg)*), $offset),
        )
    };
}

#[macro_export]
macro_rules! create_spec {
    // default
    ($name:ident ($data:ident, $params:ident : $params_t:ty) $code:block) => {
        impl $crate::specs::Spec for $name {
            type Params = $params_t;
            fn read_all<S: ByteSource>($data: &S, $params: Self::Params) -> SResult<(Self, usize)> {
                $code
            }
        }
    };
    // multiple params
    ($name:ident ($data:ident $(, $params:ident : $params_t:ty)* $(,)?) $code:block) => {
        impl $crate::specs::Spec for $name {
            type Params = ($( $params_t, )*);
            fn read_all<S: ByteSource>($data: &S, ($( $params, )*): Self::Params) -> SResult<(Self, usize)> {
                $code
            }
        }
    };
}