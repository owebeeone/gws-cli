
pub(crate) const TAG_AFTER: &str = "\
Examples:
  gwz tag v1                      create v1 across members (and the committed root)
  gwz tag v1 -m \"release one\"     annotated tag
  gwz tag                         list local tags
  gwz tag --delete v1             delete v1 locally
  gwz tag --push v1               push v1 to each member's remote
  gwz tag --push                  push every tag
  gwz tag --fetch                 fetch tags from each member's remote
  gwz tag --list --remote origin  list tags on a remote
  gwz materialize --tag v1        check out each member's refs/tags/v1";
