use std::path::PathBuf;
use std::collections::HashMap;

use rustbreak::{FileDatabase, deser::Yaml};
use failure::Error;

use crate::frontend::Frontend;
use crate::backend::Backend;
use librepology::v1::types::Package;
use librepology::v1::api::Api;

pub struct Database(FileDatabase<HashMap<String, Vec<String>>, Yaml>);

impl Database {
    pub fn open(path: PathBuf) -> Result<Self, Error> {
        FileDatabase::from_path(path, Default::default())
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
        let new_packages = self.0
            .read(|data|{
                data.iter()
                    .map(|(package_name, known_versions)| {
                        Database::get_new_packages(package_name, known_versions, backend)
                    })
                    .collect::<Vec<Result<Vec<Package>, _>>>() // dangit, this is ugly
                    .into_iter()
                    .collect::<Result<Vec<Vec<Package>>, _>>() // uh, oh...
            })??
            .into_iter()
            .flatten()
            .collect::<Vec<Package>>();

        if commit {
            for pkg in new_packages.iter() {
                self.0
                    .write(|mut data| {
                        data.entry(pkg.name().to_string())
                            .or_insert(vec![])
                            .push(pkg.version().to_string());
                    })?
            }

            let _ = self.0.save().map_err(Error::from)?;
        }

        frontend.list_packages(new_packages)
    }

    pub fn show(&self, frontend: &Frontend) -> Result<(), Error> {
        unimplemented!()
    }

    pub fn add_package(&mut self, package_name: &str, backend: &Backend) -> Result<(), Error> {
        unimplemented!()
    }
}