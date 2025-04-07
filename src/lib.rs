pub mod cli;
pub mod data;

#[cfg(test)]
mod tests {
    // This will include all tests from the tests directory
    include!("tests/data_tests.rs");
}
