#[cfg(test)]
mod test {
    use super::super::to_string;
    use indexmap::IndexMap;
    use serde::Serialize;

    #[test]
    fn foo() {
        let str = String::from("Hello World!");

        let ser = to_string(&str).unwrap();

        assert_eq!(ser, String::from("\"Hello World!\""));
    }

    #[test]
    fn simple_struct() {
        #[derive(Serialize)]
        struct Foo {
            a: i64,
            b: String,
            c: bool,
        }

        let foo = Foo {
            a: 42,
            b: String::from("Hi!"),
            c: true,
        };

        let foo_str = to_string(&foo).unwrap();

        #[rustfmt::skip]
        let expected = concat!(
            "{\n",
            "  a = 42;\n",
            "  b = \"Hi!\";\n",
            "  c = true;\n",
            "}"
        );

        assert_eq!(foo_str, expected);
    }

    #[test]
    fn map() {
        let map: IndexMap<String, bool> = [("enable", true), ("with-setting", true)]
            .iter()
            .map(|(a, b)| (a.to_string(), *b))
            .collect();

        let map_str = to_string(&map).unwrap();

        #[rustfmt::skip]
        let expected = concat!(
            "{\n",
            "  enable = true;\n",
            "  with-setting = true;\n",
            "}"
        );

        assert_eq!(map_str, expected);
    }

    #[test]
    fn nested() {
        #[derive(Serialize, Clone)]
        struct Book {
            name: String,
            author: String,
            read: bool,
        }

        let book_one = Book {
            name: "Eragon".into(),
            author: "Christopher Paolini".into(),
            read: true,
        };

        let book_two = Book {
            name: "Eldest".into(),
            author: "Christopher Paolini".into(),
            read: true,
        };

        let book_three = Book {
            name: "Brisngr".into(),
            author: "Christopher Paolini".into(),
            read: true,
        };

        let book_four = Book {
            name: "Inheritance".into(),
            author: "Christopher Paolini".into(),
            read: true,
        };

        let book_five = Book {
            name: "Murtagh".into(),
            author: "Christopher Paolini".into(),
            read: true,
        };

        let library: IndexMap<String, Book> =
            vec![book_one, book_two, book_three, book_four, book_five]
                .iter()
                .map(|b| (b.name.clone(), b.clone()))
                .collect();

        let libraries = vec![library];

        let libraries_str = to_string(&libraries).unwrap();

        println!("{libraries_str}");

        let expected = concat!(
            "[\n",
            "  {\n",
            "    Eragon = {\n",
            "      name = \"Eragon\";\n",
            "      author = \"Christopher Paolini\";\n",
            "      read = true;\n",
            "    };\n",
            "    Eldest = {\n",
            "      name = \"Eldest\";\n",
            "      author = \"Christopher Paolini\";\n",
            "      read = true;\n",
            "    };\n",
            "    Brisngr = {\n",
            "      name = \"Brisngr\";\n",
            "      author = \"Christopher Paolini\";\n",
            "      read = true;\n",
            "    };\n",
            "    Inheritance = {\n",
            "      name = \"Inheritance\";\n",
            "      author = \"Christopher Paolini\";\n",
            "      read = true;\n",
            "    };\n",
            "    Murtagh = {\n",
            "      name = \"Murtagh\";\n",
            "      author = \"Christopher Paolini\";\n",
            "      read = true;\n",
            "    };\n",
            "  }\n",
            "]"
        );

        assert_eq!(libraries_str, expected);
    }

    #[test]
    fn string() {
        let test = String::from("${} $ \" \t \n ' \\");

        let test_str = to_string(&test).unwrap();

        let expected = String::from("\"$${} $ \\\" \\t \\n ' \\\\\"");

        assert_eq!(test_str, expected);
    }

    #[test]
    fn multiline_string() {
        #[rustfmt::skip]
        let longer = concat!(
            "Lorem ipsum 'dolor ''sit amet,\n",
            "consectetur adipiscing elit,\n",
            "seddo eiusmod \ttempor incididunt\n",
            "ut labore et dolore magnam\n",
            "aliquam ${} \" quaerat voluptatem. Ut\n",
            "enim aeque doleamus animo, cum\n",
            "corpore $ dolemus, \\ fieri tamen\n",
            "permagna accessio potest, si\n",
            "aliquod aeternum et infinitum\n",
            "impendere malum nobis opinemur.\n",
            "Quod idem licet transferre in\n",
            "voluptatem, ut.",
        );

        let longer_test = to_string(&longer).unwrap();

        let expected = concat!(
            "''\n",
            "  Lorem ipsum 'dolor '''sit amet,\n",
            "  consectetur adipiscing elit,\n",
            "  seddo eiusmod \ttempor incididunt\n",
            "  ut labore et dolore magnam\n",
            "  aliquam $${} \" quaerat voluptatem. Ut\n",
            "  enim aeque doleamus animo, cum\n",
            "  corpore $ dolemus, \\ fieri tamen\n",
            "  permagna accessio potest, si\n",
            "  aliquod aeternum et infinitum\n",
            "  impendere malum nobis opinemur.\n",
            "  Quod idem licet transferre in\n",
            "  voluptatem, ut.\n",
            "''",
        );

        println!("{longer_test}");

        assert_eq!(longer_test, expected);
    }

    #[test]
    fn none() {
        #[derive(Serialize)]
        struct OptionalVals {
            a: Option<i32>,
            b: Option<i32>,
        }

        let none = OptionalVals {
            a: Some(32),
            b: None,
        };

        let none_test = to_string(&none).unwrap();

        #[rustfmt::skip]
        let expected = concat!(
            "{\n",
            "  a = 32;\n",
            "}",
        );

        assert_eq!(none_test, expected);
    }

    #[test]
    fn none_map() {
        let mut none_map: IndexMap<i32, Option<i32>> = IndexMap::new();

        none_map.insert(1, Some(1));
        none_map.insert(2, None);
        none_map.insert(3, Some(3));

        let none_map_test = to_string(&none_map).unwrap();

        #[rustfmt::skip]
        let expected = concat!(
            "{\n",
            "  1 = 1;\n",
            "  3 = 3;\n",
            "}",
        );

        assert_eq!(none_map_test, expected);
    }

    #[test]
    fn newtype_var() {
        #[derive(Serialize)]
        enum Test {
            Inches(u8),
        }

        let newtype_var = Test::Inches(8);

        let newtype_var_test = to_string(&newtype_var).unwrap();

        let expected = String::from("{ inches = 8; }");

        assert_eq!(newtype_var_test, expected);
    }

    #[test]
    fn recipe() {
        use super::super::super::content::{Content, Directory, File};
        use super::super::super::Recipe;

        let recipe = Recipe {
            name: "Test".into(),
            description: "This is a test recipe.".into(),
            languages: vec!["\x1b[38;2;0;0;0mText\x1b[0m".into()],
            contents: vec![
                Content::File(File {
                    name: "example.txt".into(),
                    content: "Hello world!".into(),
                }),
                Content::Directory(Directory {
                    name: "examples".into(),
                    files: vec![Content::File(File {
                        name: "example2.txt".into(),
                        content: "Goodbye world!".into(),
                    })],
                }),
            ],
        };

        let recipe_test = to_string(&recipe).unwrap();

        #[rustfmt::skip]
        let expected = concat!(
            "{",                                           "\n",
            "  name = \"Test\";",                          "\n",
            "  description = \"This is a test recipe.\";", "\n",
            "  languages = [",                             "\n",
            "    \"\\x1b[38;2;0;0;0mText\\x1b[0m\"",       "\n",
            "  ];",                                        "\n",
            "  contents = [",                              "\n",
            "    {",                                       "\n",
            "      name = \"example.txt\";",               "\n",
            "      content = \"Hello world!\";",           "\n",
            "    }",                                       "\n",
            "    {",                                       "\n",
            "      name = \"examples\";",                  "\n",
            "      files = [",                             "\n",
            "        {",                                   "\n",
            "          name = \"example2.txt\";",          "\n",
            "          content = \"Goodbye world!\";",     "\n",
            "        }",                                   "\n",
            "      ];",                                    "\n",
            "    }",                                       "\n",
            "  ];",                                        "\n",
            "}"
        );

        assert_eq!(recipe_test, expected);
    }
}
