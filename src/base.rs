use crate::data::Repository;

pub fn write_tree(repo: &Repository) -> Result<String, String> {
    let tree = repo.get_object("tree")?;
    Ok(String::from_utf8(tree).unwrap())
}
