use filters::filter::Filter;
use filters::ops::and::And;
use filters::ops::bool::Bool;
use filters::ops::not::Not;

use crate::config::Configuration;

struct DenyListFilter {
    repo_name: String,
}

impl DenyListFilter {
    pub fn new(repo_name: String) -> Self {
        DenyListFilter { repo_name }
    }
}

impl Filter<String> for DenyListFilter {
    fn filter(&self, element: &String) -> bool {
        element != self.repo_name
    }
}

struct AllowListFilter {
    repo_name: String,
}

impl Filter<String> for AllowListFilter {
    fn filter(&self, element: &String) -> bool {
        element == self.repo_name
    }
}

pub fn repo_filter(config: &Configuration) -> Box<Filter<String>> {
    let denylist = config
        .denylist()
        .iter()
        .cloned()
        .map(DenyListFilter::new)
        .fold(Box::new(Bool::new(true)), |accu, element| accu.and(element));
    let allowlist = config
        .allowlist()
        .iter()
        .cloned()
        .map(AllowListFilter::new)
        .fold(Box::new(Bool::new(true)), |accu, element| accu.and(element));

    Box::new(denylist.not().or(allowlist))
}
