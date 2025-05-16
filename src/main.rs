use std::fs::{write, read_to_string};
use std::env::set_current_dir;
use std::path::Path;

use regex::Regex;
use serde::{Serialize, Deserialize};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cwd = args.get(1).expect("Please provide a directory");
    set_current_dir(cwd.as_str())
        .expect("Couldn't change directories");
    let ssi = StaticSiteInfo::read_from_file("./static_site.json");
    let ppi = StaticSiteInfo::preprocess_all(&ssi);
    StaticSiteInfo::write_all(ppi);
}

pub struct OutputFile {
    name: String,
    contents: String
}

#[derive(Serialize,Deserialize)]
pub struct StaticSiteInfo {
    pub assets: Vec<StaticSiteAsset>,
    pub files: Vec<StaticSiteFile>
}
impl StaticSiteInfo {
    pub fn read_from_file(fname: &str) -> Self {
        let fcont = read_to_string(fname)
            .expect(format!("Couldn't read `{fname}`").as_str());
        serde_json::from_str(fcont.as_str())
            .expect("Couldn't deserialize StaticSiteInfo")
    }
    pub fn preprocess_all(&self) -> Vec<OutputFile> {
        let mut oup = Vec::new();
        for f in &self.files {
            let pps = StaticSiteFile::preprocess(&f, &self.assets);
            Vec::push(
                &mut oup,
                OutputFile {
                    name: f.file.clone(),
                    contents: pps
                }
            );
        }
        oup
    }
    pub fn write_all(files: Vec<OutputFile>) {
        std::fs::create_dir_all("./static_site_out/")
            .expect("Couldn't create ./static_site_out");
        for f in files {
            let path = Path::new(&f.name).parent()
                .expect(format!("Couldn't get parent of `{}`", f.name).as_str());
            std::fs::create_dir_all(format!("./static_site_out/{}", path.display()))
                .expect("Couldn't create dir");
            write(
                format!("./static_site_out/{}", f.name),
                f.contents
            ).expect(format!("Couldn't write out `{}`", f.name).as_str());
        }
    }
}

#[derive(Serialize,Deserialize)]
pub struct StaticSiteAsset {
    pub name: String,
    pub file: StaticSiteFile
}
impl StaticSiteAsset {
    pub fn read(&self) -> String {
        read_to_string(&self.file.file)
            .expect(format!("Couldn't read `{}", self.file.file).as_str())
    }
}

#[derive(Serialize,Deserialize)]
pub struct StaticSiteFile {
    pub file: String,
    pub script: Option<String>,
    pub style: Option<String>
}
impl StaticSiteFile {
    fn read(&self) -> String {
        read_to_string(&self.file)
            .expect(format!("Couldn't read `{}`", self.file).as_str())
    }
    pub fn preprocess(&self, assets: &Vec<StaticSiteAsset>) -> String {
        let mut contents = self.read();
        let mut changed;

        loop {
            let mut lines = Vec::new();
            changed = false;

            for line in contents.split_terminator('\n') {
                let res = PPXDirective::parse(line);
                match res {
                    Some(ppd) => {
                        Vec::push(
                            &mut lines,
                            PPXDirective::run(ppd, &self, &assets)
                        );
                        changed = true;
                    },
                    None => Vec::push(&mut lines, line.to_string())
                };
            }
            contents = lines.join("\n");
            if !changed {break;}
        }
        contents
    }
}

struct PPXDirective {
    prefix: String, // anything before the comment?
    command: String,
    args: Vec<String>,
    suffix: String // anything after the comment?
}
impl PPXDirective {
    fn parse(line: &str) -> Option<Self> {
        // Thank you robotic overlord for this regex
        let re = Regex::new(r"(.*?)<!--\s*@@(\w+)\s*([^@]*)\s*-->(.*)")
            .expect("Couldn't compile regex");

        re.captures(line).map(|captures| {
            let prefix = captures.get(1).map_or(
                "",
                |m| m.as_str()
            ).trim().to_string();
            let command = captures.get(2).map_or(
                "",
                |m| m.as_str()
            ).to_string();
            let args_str = captures.get(3).map_or(
                "",
                |m| m.as_str()
            ).trim();
            let suffix = captures.get(4).map_or(
                "",
                |m| m.as_str()
            ).trim().to_string();

            let args: Vec<String> = args_str.split_whitespace()
                .map(|s| s.to_string())
                .collect();

            Self {
                prefix,
                command,
                args,
                suffix
            }
        })
    }
    fn get_asset(
        name: String,
        assets: &Vec<StaticSiteAsset>
    ) -> String {
        let mut outf: Option<String> = None;
        for asset in assets {
            if asset.name == name {
                outf = Some(asset.file.file.clone());
            }
        }
        match outf {
            Some(f) => f,
            None => panic!("No asset `{}`", name)
        }
    }
    fn run(
        self,
        file: &StaticSiteFile,
        assets: &Vec<StaticSiteAsset>
    ) -> String {
        let ppstr = match self.command.as_str() {
            "STATICSTYLECOPY" => {
                match &file.style {
                    Some(style) => {
                        let s = read_to_string(style)
                        .expect(format!(
                            "Couldn't read style: `{}`",
                            style
                        ).as_str());
                        format!("<!-- {style} -->\n<style>\n{s}\n</style>\n<!-- END {style} -->\n")
                    },
                    None => panic!("Style not found for: `{}`", &file.file)
                }
            },
            "STATICSCRIPTCOPY" => {
                match &file.script {
                    Some(script) => {
                        let s = read_to_string(script)
                        .expect(format!(
                            "Couldn't read script: `{}`",
                            script
                        ).as_str());
                        format!("<!-- {script} -->\n<script>\n{s}\n</script>\n<!-- END {script} -->\n")
                    },
                    None => panic!("Script not found for: `{}`", &file.file)
                }
            },
            "STATICIMPORT" => {
                let asset_name = self.args.get(0)
                    .expect("Argument not provided for static import");
                let asset = Self::get_asset(asset_name.to_string(), assets);
                format!("<!-- {asset_name} - {asset} -->\n{}\n<!-- END {asset_name} - {asset} -->\n",
                    read_to_string(&asset)
                        .expect(format!(
                            "Couldn't read asset `{}`",
                            &asset
                        ).as_str()))
            },
            c => panic!("Unknown command: `{}`", c)
        };
        [self.prefix, ppstr, self.suffix].concat()
    }
}
