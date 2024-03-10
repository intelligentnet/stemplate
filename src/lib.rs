use std::collections::HashMap;

// Default delimiters
const START_DLIM: &str = "${";
const END_DLIM: &str = "}";

#[derive(Debug)]
/// Class to hold hidden data about template
pub struct Template<'a> {
    // Stores (key, (start, end))
    replaces: Vec<(&'a str, (usize, usize))>,
    expanded: &'a str,
    sdlim: &'a str,
    edlim: &'a str
}

/// Class implementation
impl <'a> Template<'a> {
    /// Create a new template using a string containing ${..} variables
    /// Note: will only dereference 8 nested levels of variables
    /// Simple default value;
    /// # Example
    /// ```
    /// use stemplate::Template;
    /// Template::new("My name is ${name}");
    /// ```
    /// # Example
    /// ```
    /// use stemplate::Template;
    /// // Simple default value (variable not supplied in template or is empty";
    /// Template::new("My name is ${name:-Fred}");
    /// ```
    /// # Example
    /// ```
    /// use stemplate::Template;
    /// // Nested variables where fullname = "${first:=Fred} ${last:=Bloggs}"
    /// Template::new("My name is ${fullname}");
    /// ```
    pub fn new(expanded: &'a str) -> Self {
        Template::new_delimit(expanded, START_DLIM, END_DLIM)
    }

    /// Create a new template as above but choose different delimiters
    /// # Example
    /// use stemplate::Template;
    /// Template::new("My name is {%name%}", "{%", "%}");
    pub fn new_delimit(expanded: &'a str, sdlim: &'a str, edlim: &'a str) -> Self {
        let expanded = expanded.trim();
        let mut template = Self { replaces: Vec::new(), expanded, sdlim, edlim };

        if expanded.is_empty() {
            return template;
        }

        let replaces = &mut template.replaces;

        // Current position in the format string
        let mut cursor = 0;

        while cursor <= expanded.len() {
            if let Some(start) = expanded[cursor..].find(sdlim) {
                let start = start + cursor;
                if let Some(end) = expanded[start..].find(edlim) {
                    let end = end + start;
                    replaces.push((
                        // The extracted key
                        &expanded[(start + sdlim.len())..end],
                        (start, (end + edlim.len())),
                    ));

                    // Move cursor to the end of this match
                    cursor = end + edlim.len();
                } else {
                    // Assume part of the text
                    break;
                }
            } else {
                replaces.push((
                    // The extracted key
                    &expanded[cursor..cursor], (cursor, cursor),
                ));
                break;
            }
        }
        template
    }

    /// Render a template.
    /// # Example
    /// ```
    /// use std::collections::HashMap;
    /// use stemplate::Template;
    /// let template = Template::new("My name is ${name}");
    /// let mut args = HashMap::new();
    /// args.insert("name", "Fred");
    /// let s = template.render(&args);
    /// assert_eq!(s, "My name is Fred");
    /// ```
    pub fn render<V: AsRef<str> + std::fmt::Debug + std::string::ToString>(&self, vars: &HashMap<&str, V>) -> String {
        self.recursive_render(vars, 0)
    }

    /// Render a template with string values. Convenience for use with serde hash maps.
    /// # Example
    /// ```
    /// use std::collections::HashMap;
    /// use stemplate::Template;
    /// let template = Template::new("My name is ${name}");
    /// let mut args = HashMap::new();
    /// args.insert("name", "Fred");
    /// let s = template.render(&args);
    /// assert_eq!(s, "My name is Fred");
    /// ```
    /// # Example
    /// ```
    /// // Nested variables
    /// use std::collections::HashMap;
    /// use stemplate::Template;
    /// let mut args = HashMap::new();
    /// args.insert("first", "Doris");
    /// args.insert("fullname", "${first:=Fred} ${last:=Bloggs}");
    /// let template = Template::new("${fullname}");
    /// let s = template.render(&args);
    /// assert_eq!(s, "Doris Bloggs");
    /// ```
    pub fn render_strings(&self, vars: &HashMap<String, String>) -> String {
        let vars: HashMap<&str, &str> = vars.iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        
        self.recursive_render(&vars, 0)
    }

    /// Render a template from environment variables.
    /// # Example
    /// ```
    /// use stemplate::Template;
    /// // Using Googles LLM API. GEMINI_URL contains other env variables
    /// let url: String = Template::new_delimit("{GEMINI_URL}", "{", "}").render_env();
    /// ```
    /// # Example
    /// ```
    /// use stemplate::Template;
    /// let s = Template::new("File contains: ${!test.inc}").render_env();
    /// //assert_eq!(s, "File contains: inc");
    /// ```
    pub fn render_env(&self) -> String {
        let vars: HashMap<&str, String> = HashMap::new();

        self.recursive_render(&vars, 0)
    }

    fn recursive_render<V: AsRef<str> + std::fmt::Debug + std::string::ToString>(&self, vars: &HashMap<&str, V>, level: u8) -> String {

        fn default<V: AsRef<str> + std::fmt::Debug + std::string::ToString>(key: &str, delimiter: &str, vars: &HashMap<&str, V>) -> String {
            let bits: Vec<_> = key.split(delimiter).collect();

            match vars.get(bits[0]) {
                Some(v) if !v.as_ref().is_empty() => 
                   v.to_string(),
                _ => {
                   match std::env::var(bits[0]) {
                       Ok(v) => v,
                       Err(_) => bits[1].to_string()
                   }
                }
            }
        }

        let replaces = &self.replaces;
        let expanded = &self.expanded;
        let mut output = String::new();
        let mut cursor: usize = 0;

        for (key, (start, end)) in replaces.iter() {
            output.push_str(&expanded[cursor..*start]);
            // Unwrapping is be safe at this point
            match vars.get(key) {
                Some(v) => {
                    output.push_str(v.as_ref())
                },
                None => {
                    // Implement default values if provided
                    if key.contains(":-") {
                        output.push_str(&default(key, ":-", vars));
                    } else if key.contains(":=") {
                        output.push_str(&default(key, ":=", vars));
                    } else if key.starts_with('!') && key.ends_with(".inc") {
                        match std::fs::read_to_string(&key[1..]) {
                            Ok(content) => {
                                let content = content.trim();
                                if content.contains(self.sdlim) {
                                    let content = Template::new_delimit(&content, self.sdlim, self.edlim).recursive_render(vars, level + 1);
                                    output.push_str(content.trim().as_ref())
                                } else {
                                    output.push_str(content.trim().as_ref())
                                }
                            },
                            Err(_) => output.push_str("".as_ref())
                        }
                    } else {
                        match std::env::var(key) {
                            Ok(v) => output.push_str(v.as_ref()),
                            Err(_) => output.push_str("".as_ref())
                        }
                    }
                }
            }
            cursor = *end;
        }

        // If there's more text after the `${}`
        if cursor < expanded.len() {
            output.push_str(&expanded[cursor..]);
        }

        if level < 8 && output.contains(self.sdlim) {
            output = Template::new_delimit(&output, self.sdlim, self.edlim).recursive_render(vars, level + 1);
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn once() {
        let test: &str = "Hello, ${name}, nice to meet you.";
        let mut args = HashMap::new();
        args.insert("name", "Charles");

        let s = Template::new(test).render(&args);

        assert_eq!(s, "Hello, Charles, nice to meet you.");
    }

    #[test]
    fn beginning() {
        let test: &str = "${plural capitalized food} taste good.";
        let mut args = HashMap::new();
        args.insert("plural capitalized food", "Apples");

        let s = Template::new(test).render(&args);

        assert_eq!(s, "Apples taste good.");
    }

    #[test]
    fn only() {
        let test: &str = "${why}";
        let mut args = HashMap::new();
        args.insert("why", "would you ever do this");

        let s = Template::new(test).render(&args);

        assert_eq!(s, "would you ever do this");
    }

    #[test]
    fn end() {
        let test: &str = "I really love ${something}";
        let mut args = HashMap::new();
        args.insert("something", "programming");

        let s = Template::new(test).render(&args);

        assert_eq!(s, "I really love programming");
    }

    #[test]
    fn empty() {
        let test: &str = "";
        let args:HashMap<&str, &str> = HashMap::new();

        let s = Template::new(test).render(&args);

        assert_eq!(s, "");
    }

    #[test]
    fn two() {
        let test: &str = "Hello, ${name}. You remind me of another ${name}.";
        let mut args = HashMap::new();
        args.insert("name", "Charles");

        let s = Template::new(test).render(&args);

        assert_eq!(s, "Hello, Charles. You remind me of another Charles.");
    }

    #[test]
    fn twice() {
        let test: &str = "${name}, why are you writing code at ${time} again?";
        let mut args = HashMap::new();
        args.insert("name", "Charles");
        args.insert("time", "2 AM");

        let s = Template::new(test).render(&args);

        assert_eq!(s, "Charles, why are you writing code at 2 AM again?");
    }

    #[test]
    fn default_empty() {
        let test: &str = "${name:-Henry}, why are you writing code at ${time} again?";
        let mut args = HashMap::new();
        //args.insert("name", "Charles");
        args.insert("time", "2 AM");

        let s = Template::new(test).render(&args);

        assert_eq!(s, "Henry, why are you writing code at 2 AM again?");
    }

    #[test]
    fn default_some() {
        let test: &str = "${name:-Henry}, why are you writing code at ${time} again?";
        let mut args = HashMap::new();
        args.insert("name", "Charles");
        args.insert("time", "2 AM");

        let s = Template::new(test).render(&args);

        assert_eq!(s, "Charles, why are you writing code at 2 AM again?");
    }

    #[test]
    fn recursive_empty() {
        let test: &str = "${name:-Henry}, why are you writing code at ${time} again?";
        let mut args = HashMap::new();
        args.insert("name", "${king:-Big Man}");
        args.insert("time", "2 AM");

        let s = Template::new(test).render(&args);

        assert_eq!(s, "Big Man, why are you writing code at 2 AM again?");
    }

    #[test]
    fn recursive_some() {
        let test: &str = "${name:-Henry}, why are you writing code at ${time} again?";
        let mut args = HashMap::new();
        args.insert("king", "William");
        args.insert("name", "${king:-Big Man}");
        args.insert("time", "2 AM");

        let s = Template::new(test).render(&args);

        assert_eq!(s, "William, why are you writing code at 2 AM again?");
    }

    #[test]
    fn from_env() {
        let test: &str = "My name is ${NAME}";
        let s = Template::new(test).render_env();

        assert_eq!(s, "My name is ");

        std::env::set_var("NAME", "Henry");

        let s = Template::new(test).render_env();

        assert_eq!(s, "My name is Henry");
    }

    #[test]
    fn alone() {
        let mut args = HashMap::new();
        args.insert("dog", "woofers");

        let s = Template::new("${dog}").render(&args);

        assert_eq!(s, "woofers");
    }

    #[test]
    fn alt_delimeters() {
        let mut args = HashMap::new();
        args.insert("dog", "woofers");
        args.insert("cat", "{cat_name:=moggy} that says {cat_noise}");
        args.insert("cat_noise", "meeow");

        let s = Template::new_delimit("My dog {dog} has a friend {cat}", "{", "}").render(&args);

        assert_eq!(s, "My dog woofers has a friend moggy that says meeow");
    }

    #[test]
    fn include() {
        let mut args = HashMap::new();
        args.insert("example", "text");
        let s = Template::new("File contains: ${!test.inc}").render(&args);

        assert_eq!(s, "File contains: inc text");
    }

    #[test]
    fn dont_include() {
        let s = Template::new("${!/etc/passwd}").render_env();

        assert_eq!(s, "");
    }
}
