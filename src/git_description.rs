use std::path::Path;

use gix::commit::describe::SelectRef;

pub struct GitDescription {
    pub parent_tag_name: Option<String>,
    pub on_tag: bool,
}

fn retrieve_description_inner(repo_path: &Path) -> anyhow::Result<GitDescription> {
    let repo = gix::discover(repo_path)?;
    let mut head = repo.head()?;
    let head_commit = head.peel_to_commit_in_place()?;
    let describe = head_commit
        .describe()
        .names(SelectRef::AllTags)
        .id_as_fallback(false);
    let Some(resolution) = describe.try_resolve()? else {
        return Ok(GitDescription {
            parent_tag_name: None,
            on_tag: false,
        });
    };
    // should be save because we don't ask for a fallback
    let tag_name = resolution.outcome.name.unwrap();

    // now resolve the tag name back to a commit id
    // it would be nice if we didn't have to go through this additional step
    let ref_name = format!("tags/{}", tag_name);
    let mut tag_ref = repo.find_reference(&ref_name)?;
    let tag_id = tag_ref.peel_to_id_in_place()?;

    Ok(GitDescription {
        parent_tag_name: Some(tag_name.to_string()),
        on_tag: tag_id == head.id().unwrap(),
    })
}

pub fn retrieve_description(repo_path: &Path) -> Result<GitDescription, crate::Error> {
    retrieve_description_inner(repo_path).map_err(crate::Error::Git)
}
