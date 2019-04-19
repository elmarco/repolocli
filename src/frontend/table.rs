use std::io::Stdout;
use std::ops::Deref;

use librepology::v1::types::Package;
use librepology::v1::types::Problem;
use failure::Fallible as Result;
use prettytable::format;
use prettytable::Table;

use crate::frontend::Frontend;

pub struct TableFrontend(Stdout);

impl TableFrontend {
    pub fn new(stdout: Stdout) -> Self {
        TableFrontend(stdout)
    }
}

impl Frontend for TableFrontend {
    fn list_packages(&self, packages: Vec<Package>) -> Result<()> {
        let mut table = Table::new();
        let format = format::FormatBuilder::new()
            .column_separator('|')
            .borders('|')
            .separators(
                &[format::LinePosition::Title, format::LinePosition::Top, format::LinePosition::Bottom],
                format::LineSeparator::new('-', '+', '+', '+')
            )
            .padding(1, 1)
            .build();
        table.set_format(format);

        table.set_titles(row!["Name", "Version", "Repo", "Status", "URL"]);

        packages.iter().for_each(|package| {
            let status = if let Some(stat) = package.status() {
                format!("{}", stat)
            } else {
                String::from("No status")
            }; // not optimal, but works for now.

            let url = if let Some(url) = package.www() {
                if let Some(url) = url.first() {
                    format!("{}", url.deref())
                } else {
                    String::from("")
                }
            } else {
                String::from("")
            }; // not optimal, but works for now

            table.add_row(row![package.name(), package.version(), package.repo(), status, url]);
        });

        let mut outlock = self.0.lock();
        table.print(&mut outlock)?;

        Ok(())
    }

    fn list_problems(&self, problems: Vec<Problem>) -> Result<()> {
        let mut table = Table::new();
        let format = format::FormatBuilder::new()
            .column_separator('|')
            .borders('|')
            .separators(
                &[format::LinePosition::Title, format::LinePosition::Top, format::LinePosition::Bottom],
                format::LineSeparator::new('-', '+', '+', '+')
            )
            .padding(1, 1)
            .build();
        table.set_format(format);

        table.set_titles(row!["Repo", "Name", "EffName", "Maintainer", "Description"]);

        problems.iter().for_each(|problem| {
            trace!("Adding row for: {:?}", problem);
            table.add_row(row![
                problem.repo(),
                problem.name(),
                problem.effname(),
                problem.maintainer(),
                problem.problem_description()
            ]);
        });

        let mut outlock = self.0.lock();
        table.print(&mut outlock)?;

        Ok(())
    }
}
