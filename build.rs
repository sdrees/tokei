extern crate handlebars;
extern crate ignore;
extern crate serde_json;

use std::ffi::OsStr;
use std::fs::{self, File};
use std::path::Path;
use std::{cmp, env, error};

use handlebars::Handlebars;
use ignore::Walk;
use serde_json::Value;

fn main() -> Result<(), Box<dyn error::Error>> {
    let out_dir = env::var_os("OUT_DIR").expect("No OUT_DIR variable.");
    generate_languages(&out_dir)?;
    generate_tests(&out_dir)?;

    Ok(())
}

fn generate_languages(out_dir: &OsStr) -> Result<(), Box<dyn error::Error>> {
    let handlebars = {
        let mut h = Handlebars::new();
        h.register_escape_fn(handlebars::no_escape);
        h
    };

    let mut json: Value = serde_json::from_reader(File::open(&"languages.json")?)?;

    for (_key, ref mut item) in json
        .get_mut("languages")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .iter_mut()
    {
        macro_rules! sort_prop {
            ($prop:expr) => {{
                if let Some(ref mut prop) = item.get_mut($prop) {
                    prop.as_array_mut()
                        .unwrap()
                        .sort_unstable_by(compare_json_str_len)
                }
            }};
        }

        sort_prop!("quotes");
        sort_prop!("multi_line");
    }

    let output = Path::new(&out_dir).join("language_type.rs");
    let mut source_template = File::open(&"src/language/language_type.hbs.rs")?;
    let mut output_file = File::create(&output)?;

    handlebars.render_template_source_to_write(&mut source_template, &json, &mut output_file)?;
    Ok(())
}

fn compare_json_str_len(a: &Value, b: &Value) -> cmp::Ordering {
    let a = a.as_array().expect("a as array");
    let b = b.as_array().expect("b as array");

    let max_a_size = a.iter().map(|e| e.as_str().unwrap().len()).max().unwrap();
    let max_b_size = b.iter().map(|e| e.as_str().unwrap().len()).max().unwrap();

    max_b_size.cmp(&max_a_size)
}

fn generate_tests(out_dir: &OsStr) -> Result<(), Box<dyn error::Error>> {
    // Length of string literal below by number of languages
    const INITIAL_BUFFER_SIZE: usize = 989 * 130;
    let mut string = String::with_capacity(INITIAL_BUFFER_SIZE);

    let walker = Walk::new("./tests/data/").filter(|p| match p {
        Ok(ref p) => {
            if let Ok(ref p) = p.metadata() {
                p.is_file()
            } else {
                false
            }
        }
        _ => false,
    });

    for path in walker {
        let path = path?;
        let path = path.path();

        let name = path.file_stem().unwrap().to_str().unwrap().to_lowercase();

        string.push_str(&format!(
            r#"
        #[test]
        fn {0}() {{
            let mut languages = Languages::new();
            languages.get_statistics(&["{1}"], &[], &Config::default());

            if languages.len() != 1 {{
                panic!("wrong languages detected: expected just {0}, found {{:?}}",
                       languages.into_iter().collect::<Vec<_>>());
            }}

            let (name, mut language) = languages.into_iter().next().unwrap();

            let contents = fs::read_to_string("{1}").unwrap();

            assert_eq!(get_digit!(LINES, contents), language.lines);
            println!("{{}} LINES MATCH", name);
            assert_eq!(get_digit!(CODE, contents), language.code);
            println!("{{}} CODE MATCH", name);
            assert_eq!(get_digit!(COMMENTS, contents), language.comments);
            println!("{{}} COMMENTS MATCH", name);
            assert_eq!(get_digit!(BLANKS, contents), language.blanks);
            println!("{{}} BLANKS MATCH", name);

            let stats = language.stats.pop().unwrap();

            assert_eq!(language.lines, stats.lines);
            assert_eq!(language.code, stats.code);
            assert_eq!(language.comments, stats.comments);
            assert_eq!(language.blanks, stats.blanks);
        }}
        "#,
            name,
            path.display()
        ));
    }

    Ok(fs::write(Path::new(&out_dir).join("tests.rs"), string)?)
}
