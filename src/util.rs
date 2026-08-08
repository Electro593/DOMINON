pub fn check_all<T, const N: usize>(exprs: [Option<T>; N], err: &str) -> Result<[T; N], &str> {
    exprs
        .into_iter()
        .collect::<Option<Vec<T>>>()
        .ok_or(err)?
        .try_into()
        .map_err(|_| "Internal error: size mismatch")
}
