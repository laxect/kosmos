use std::cmp::PartialEq;
use structopt::StructOpt;

#[derive(Clone, Debug, StructOpt, PartialEq)]
pub(crate) enum Item {
    Add { name: Vec<String> },
    Remove { name: Vec<String> },
    List,
    Clear,
}

#[derive(Clone, Debug, StructOpt, PartialEq)]
pub(crate) enum Request {
    Keyword(Item),
    Page(Item),
}

impl Request {
    pub(crate) fn namespace(&self) -> &'static str {
        match self {
            Request::Keyword(_) => "keyword",
            Request::Page(_) => "page",
        }
    }

    pub(crate) fn inner(self) -> Item {
        match self {
            Request::Keyword(item) => item,
            Request::Page(item) => item,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_parse() {
        let input = "aoi keyword add test";
        let input = input.split_whitespace();
        let req = Request::from_iter_safe(input).unwrap();
        let expect = Request::Keyword(Item::Add {
            name: vec!["test".to_owned()],
        });
        assert_eq!(req, expect);
    }
}
