use std::path::PathBuf;
use std::collections::HashMap;
use std::ops::Deref;

use rustbreak::{FileDatabase, deser::Yaml};
use failure::Error;
use itertools::Itertools;

use crate::frontend::Frontend;
use crate::backend::Backend;
use librepology::v1::types::Package;
use librepology::v1::api::Api;

pub struct Database(FileDatabase<HashMap<String, Vec<String>>, Yaml>);

impl Database {
    pub fn open(path: PathBuf) -> Result<Self, Error> {
        debug!("Loading database...");
        FileDatabase::from_path(path, Default::default())
            .and_then(|fdb| fdb.load().map(|_| fdb))
            .map_err(Error::from)
            .map(Database)
    }

    fn get_new_packages(package_name: &String, known_versions: &Vec<String>, backend: &Backend) -> Result<Vec<Package>, Error> {
        let new_versions = backend
            .project(package_name)?
            .into_iter()
            .filter(|pkg| !known_versions.contains(pkg.version()))
            .collect();

        Ok(new_versions)
    }

    pub fn update(&mut self, commit: bool, backend: &Backend, frontend: &Frontend) -> Result<(), Error> {
        let package_names : Vec<String> = self.0.read(|data|  {
            data.keys().cloned().collect()
        })?;

        debug!("Updating: {:?}", package_names);

        package_names
            .into_iter()
            .map(|pkgname| {
                let known_versions = self.0
                    .read(|data|{
                        if let Some(v) = data.get(&pkgname).map(Vec::clone) {
                            v
                        } else {
                            vec![]
                        }
                    })?;

                debug!("Known versions for {}: {:?}", pkgname, known_versions);

                let new_upstream_packages : Vec<Package> = backend
                    .project(&pkgname)?
                    .into_iter()
                    .filter(|pkg| !known_versions.contains(pkg.version().deref()))
                    .collect();

                debug!("Found {} new upstream packages for {}", new_upstream_packages.len(), pkgname);

                if commit {
                    self.0
                        .write(|data| {
                            let mut new_versions = new_upstream_packages
                                .iter()
                                .map(|pkg| pkg.version().to_string())
                                .collect();
                            data.entry(pkgname).or_insert(vec![])
                                .append(&mut new_versions);
                        })?;
                }

                debug!("Listing...");
                frontend.list_packages(new_upstream_packages)?;

                Ok(())
            })
            .collect::<Result<(), Error>>();

        if commit {
            let _ = self.0.save().map_err(Error::from)?;
        }

        Ok(())
    }

    pub fn show(&self, frontend: &Frontend, backend: &Backend) -> Result<(), Error> {
        let packages = self.0
            .read(|d| d.keys().cloned().collect::<Vec<String>>())?
            .iter()
            .map(|name| {debug!("Found: {}", name); name})
            .map(|name| backend.project(name))
            .collect::<Vec<Result<_, _>>>() // ugly as
            .into_iter()
            .collect::<Result<Vec<Vec<_>>, _>>()? // hell!
            .into_iter()
            .flatten()
            .collect();

        debug!("Listing the following packages: {:?}", packages);

        frontend.list_packages(packages)
    }

    pub fn add_package(&mut self, package_name: &str, backend: &Backend) -> Result<(), Error> {
        debug!("Adding package: '{}'", package_name);
        let mut versions = backend.project(package_name)?
            .iter()
            .map(|p| p.version().to_string())
            .unique()
            .collect();

        debug!("Adding the following versions for {} to the database: {:?}", package_name, versions);

        self.0
            .write(|data| {
                data.entry(String::from(package_name)).or_insert(vec![]).append(&mut versions)
            })
            .map_err(Error::from)
    }
}