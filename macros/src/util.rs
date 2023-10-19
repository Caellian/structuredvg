use std::collections::VecDeque;

use syn::{Error, Result};

pub fn flatten_result_vec<T>(results: Vec<Result<T>>) -> Result<Vec<T>> {
    if results.iter().any(|it| it.is_err()) {
        let mut errors: VecDeque<Error> = results.into_iter().filter_map(Result::err).collect();
        let mut result = errors.pop_front().unwrap();
        for other in errors {
            result.combine(other)
        }
        Err(result)
    } else {
        Ok(results.into_iter().filter_map(Result::ok).collect())
    }
}
