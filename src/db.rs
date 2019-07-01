use std::path::PathBuf;
use std::collections::HashMap;
use std::collections::HashSet;

use rustbreak::{FileDatabase, deser::Yaml};
use failure::Error;
use itertools::Itertools;

use crate::frontend::Frontend;
use crate::backend::Backend;
use librepology::v1::types::Package;
use librepology::v1::api::Api;

pub struct Database(FileDatabase<HashMap<String, HashSet<String>>, Yaml>);

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
                let mut known_versions = self.0
                    .read(|data|{
                        if let Some(v) = data.get(&pkgname).map(HashSet::clone) {
                            v
                        } else {
                            HashSet::new()
                        }
                    })?;

                debug!("Known versions for {}: {:?}", pkgname, known_versions);

                backend
                    .project(&pkgname)?
                    .into_iter()
                    .for_each(|pkg| {
                        known_versions.insert(pkg.version().to_string());
                    });

                if commit {
                    self.0
                        .write(|data| {
                            data.insert(pkgname.clone(), known_versions.clone());
                        })?;
                }

                debug!("Listing...");
                let new_package_versions = known_versions.into_iter().collect();
                frontend.list_package_versions(&pkgname, new_package_versions)
            })
            .collect::<Result<(), Error>>()?;

        if commit {
            let _ = self.0.save().map_err(Error::from)?;
        }

        Ok(())
    }

    pub fn show(&self, frontend: &Frontend, _backend: &Backend) -> Result<(), Error> {
        self.0
            .read(|data| {
                data.iter()
                    .map(|(pkgname, versions)| {
                        frontend.list_package_versions(pkgname, versions.into_iter().map(String::clone).collect())
                    })
                    .collect::<Result<_, _>>()
            })?
    }

    pub fn add_package(&mut self, package_name: &str, backend: &Backend) -> Result<(), Error> {
        debug!("Adding package: '{}'", package_name);
        let versions = backend.project(package_name)?
            .iter()
            .map(|p| p.version().to_string())
            .unique()
            .collect();

        debug!("Adding the following versions for {} to the database: {:?}", package_name, versions);

        self.0
            .write(|data| {
                data.insert(String::from(package_name), versions)
            })?;

        self.0.save().map_err(Error::from)
    }
}