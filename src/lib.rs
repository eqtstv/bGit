pub mod cli;
pub mod repository;

#[cfg(test)]
mod tests {
    // This will include all tests from the tests directory
    include!("tests/repository_tests.rs");
    include!("tests/diff_tests.rs");
}
