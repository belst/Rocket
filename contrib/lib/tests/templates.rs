#![feature(plugin, decl_macro)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_contrib;

#[cfg(feature = "templates")]
mod templates_tests {
    use std::env;
    use std::path::PathBuf;

    use rocket::{Rocket, http::RawStr};
    use rocket::config::{Config, Environment};
    use rocket_contrib::{Template, TemplateMetadata};

    #[get("/<engine>/<name>")]
    fn template_check(md: TemplateMetadata, engine: &RawStr, name: &RawStr) -> Option<()> {
        match md.contains_template(&format!("{}/{}", engine, name)) {
            true => Some(()),
            false => None
        }
    }

    fn template_root() -> PathBuf {
        let cwd = env::current_dir().expect("current working directory");
        cwd.join("tests").join("templates")
    }

    fn rocket() -> Rocket {
        let config = Config::build(Environment::Development)
            .extra("template_dir", template_root().to_str().expect("template directory"))
            .expect("valid configuration");

        ::rocket::custom(config).attach(Template::fairing())
            .mount("/", routes![template_check])
    }

    #[cfg(feature = "tera_templates")]
    mod tera_tests {
        use super::*;
        use std::collections::HashMap;
        use rocket::http::Status;
        use rocket::local::Client;

        const UNESCAPED_EXPECTED: &'static str
            = "\nh_start\ntitle: _test_\nh_end\n\n\n<script />\n\nfoot\n";
        const ESCAPED_EXPECTED: &'static str
            = "\nh_start\ntitle: _test_\nh_end\n\n\n&lt;script &#x2F;&gt;\n\nfoot\n";

        #[test]
        fn test_tera_templates() {
            let rocket = rocket();
            let mut map = HashMap::new();
            map.insert("title", "_test_");
            map.insert("content", "<script />");

            // Test with a txt file, which shouldn't escape.
            let template = Template::show(&rocket, "tera/txt_test", &map);
            assert_eq!(template, Some(UNESCAPED_EXPECTED.into()));

            // Now with an HTML file, which should.
            let template = Template::show(&rocket, "tera/html_test", &map);
            assert_eq!(template, Some(ESCAPED_EXPECTED.into()));
        }

        #[test]
        fn test_template_metadata_with_tera() {
            let client = Client::new(rocket()).unwrap();

            let response = client.get("/tera/txt_test").dispatch();
            assert_eq!(response.status(), Status::Ok);

            let response = client.get("/tera/html_test").dispatch();
            assert_eq!(response.status(), Status::Ok);

            let response = client.get("/tera/not_existing").dispatch();
            assert_eq!(response.status(), Status::NotFound);

            let response = client.get("/hbs/txt_test").dispatch();
            assert_eq!(response.status(), Status::NotFound);
        }
    }

    #[cfg(feature = "handlebars_templates")]
    mod handlebars_tests {
        use super::*;
        use std::collections::HashMap;
        use rocket::http::Status;
        use rocket::local::Client;

        const EXPECTED: &'static str
            = "Hello _test_!\n\n<main> &lt;script /&gt; hi </main>\nDone.\n\n";

        #[test]
        fn test_handlebars_templates() {
            let rocket = rocket();
            let mut map = HashMap::new();
            map.insert("title", "_test_");
            map.insert("content", "<script /> hi");

            // Test with a txt file, which shouldn't escape.
            let template = Template::show(&rocket, "hbs/test", &map);
            assert_eq!(template, Some(EXPECTED.into()));
        }

        #[test]
        fn test_template_metadata_with_handlebars() {
            let client = Client::new(rocket()).unwrap();

            let response = client.get("/hbs/test").dispatch();
            assert_eq!(response.status(), Status::Ok);

            let response = client.get("/hbs/not_existing").dispatch();
            assert_eq!(response.status(), Status::NotFound);

            let response = client.get("/tera/test").dispatch();
            assert_eq!(response.status(), Status::NotFound);
        }
    }
}
