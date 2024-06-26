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
    /// Note: will only dereference 16 nested levels of variables
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
    /// Template::new_delimit("My name is {%name%}", "{%", "%}");
    pub fn new_delimit(expanded: &'a str, sdlim: &'a str, edlim: &'a str) -> Self {
        fn find_end(s: &str, sdlim: &str, edlim: &str) -> Option<usize> {
            let mut level = 0;

            for (i, c) in s.chars().enumerate() {
                if sdlim.starts_with(c) && s[i..].starts_with(sdlim) {
                    level += 1;
                } else if edlim.starts_with(c) && s[i..].starts_with(edlim) {
                    level -= 1;
                    if level == 0 {
                        return Some(i);
                    }
                }
            }

            None
        }

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
                if let Some(end) = find_end(&expanded[start..], sdlim, edlim) {
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
    /// # Example
    /// ```
    /// // Multi-valued example use in *<delimit>
    /// // Delimit is optional and is newline by default
    /// // This is useful for lists etc.
    /// // Normally HTML markup would be included
    /// use std::collections::HashMap;
    /// use stemplate::Template;
    /// let mut args = HashMap::new();
    /// args.insert("dog", "woofers|rex|freddy");
    /// args.insert("cat", "kitty|moggi");
    /// args.insert("pets", "${dog} and ${cat}");
    /// let s = Template::new("${*|pets}").render(&args);
    /// assert_eq!(s, "woofers and kitty|rex and moggi");
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

        fn other_sources<V: AsRef<str> + std::fmt::Debug + std::string::ToString>(key: &str, vars: &HashMap<&str, V>) -> String {
            // Implement default values if provided
            if key.contains(":-") {
                default(key, ":-", vars)
            } else if key.contains(":=") {
                default(key, ":=", vars)
            // Okay, try environment then
            } else {
                match std::env::var(key) {
                    Ok(v) => v.trim().into(),
                    Err(_) => "".into()
                }
            }
        }

        let replaces = &self.replaces;
        let expanded = &self.expanded;
        let mut output = String::new();
        let mut cursor: usize = 0;

        // Only used for Multi-values
        let mut mvv: HashMap<&str, Vec<String>> = HashMap::new();
        let mut vars2: HashMap<&str, String> = HashMap::new();
        let mut vc: HashMap<&str, usize> = HashMap::new();
        let mut is_single = true;
        let mut is_literal = false;

        for (key, (start, end)) in replaces.iter() {
            output.push_str(&expanded[cursor..*start]);
            // Read from file?
            if key.starts_with('!') && key.ends_with(".inc") {
                match std::fs::read_to_string(&key[1..]) {
                    Ok(content) => {
                        let mut content = content.trim().to_string();

                        if content.contains(self.sdlim) {
                            content = Template::new_delimit(&content, self.sdlim, self.edlim).recursive_render(vars, level + 1);
                        }

                        output.push_str(content.trim().as_ref())
                    },
                    Err(_) => output.push_str("".as_ref())
                }
            // Exists with value test
            } else if key.starts_with('?') && key.contains('=') {
               let mut value: String = "".to_string();
               let mut vd: Vec<&str> = key.split(":-").collect();

               if vd.len() != 2 {
                   vd = key.split(":=").collect();
               }
               if vd.len() == 2 {
                   let lhs = &(vd[0])[1..];
                   let vv: Vec<&str> = lhs.split('=').collect();

                   if vv.len() == 2 {
                       if let Some(v) = vars.get(vv[0]) {
                           if v.to_string() == vv[1] {
                               value = vd[1].trim().to_string();
                           }
                       }
                   }
                   output.push_str(value.as_ref())
               }
            // Multi Value substitution
            } else if let Some(mut key) = key.strip_prefix('*') {
                is_single = false;
                let delim = if key.chars().next().unwrap().is_alphabetic() {
                    "\n"
                } else {
                    let delim = &key[0..1];
                    key = &key[1..];

                    delim
                };
                if let Some(key) = vars.get(key) {
                    let key = key.to_string();

                    if mvv.is_empty() { // We only need to do this once
                        vars2 = vars.iter()
                            .map(|(k,v)| (*k, v.to_string()))
                            .collect();
                        for (k, v) in vars2.iter() {
                            let v = v.to_string();
                            if v.contains('|') {
                                let val = v.split('|').map(|i| i.trim().into()).collect();
                                mvv.insert(k, val);
                            }
                        }
                    }
                    let mi = mvv.iter()
                        .filter(|(k,_)| key.contains(&format!("{}{k}{}", self.sdlim, self.edlim)))
                        .map(|(_,v)| v.len())
                        .min();
                    if let Some(mi) = mi {
                        for i in 0 .. mi {
                            mvv.iter()
                                .filter(|(k,v)| mi <= v.len() && key.contains(&format!("{}{k}{}", self.sdlim, self.edlim)))
                                .for_each(|(k,v)| { vars2.insert(k, v[i].clone()); });
                            let mut content = Template::new_delimit(&key, self.sdlim, self.edlim).recursive_render(&vars2, level + 1) + delim;

                            if i == mi - 1 {
                                content = content[..content.len()-1].to_string();
                            }

                            output.push_str(content.as_ref())
                        }
                    } else {
                            let mut content = Template::new_delimit(&key, self.sdlim, self.edlim).recursive_render(&vars2, level + 1) + delim;
                            content = content[..content.len()-1].to_string();

                            output.push_str(content.as_ref())
                    }
                }
            } else if let Some(key) = key.strip_prefix('=') {
                if let Some(content) = vars.get(key) {
                    is_literal = true;
                    output.push_str(content.as_ref())
                }
            } else if let Some(key) = key.strip_prefix('#') {
                if let Some(v) = vars.get(key) {
                    let v = v.to_string();
                    let vs: Vec<&str> = v.split('|').collect();
                    let _ = vc.entry(key)
                        .and_modify(|v| { *v = (*v + 1) % vs.len(); })
                        .or_insert(0);
                    let i = vc.get(key).unwrap();
                    output.push_str(vs[*i])
                }
            } else {
                let v = 
                    match vars.get(key) {
                        Some(v) => v.to_string(),
                        None => other_sources(key, vars)
                    };

                if is_single || !v.contains('|') {
                    output.push_str(v.trim().as_ref())
                }
            }
            cursor = *end;
        }

        if !is_literal && level < 16 && output.contains(self.sdlim) {
            output = Template::new_delimit(&output, self.sdlim, self.edlim).recursive_render(vars, level + 1);
        }

        // If there's more text after the `${}`
        if cursor < expanded.len() {
            output.push_str(&expanded[cursor..]);
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
    fn two_values() {
        let test: &str = "Hello, ${#name}. You remind me of another ${#name}.";
        let mut args = HashMap::new();
        args.insert("name", "Charles|Harry");

        let s = Template::new(test).render(&args);

        assert_eq!(s, "Hello, Charles. You remind me of another Harry.");
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
    fn recursive_twice() {
        let test: &str = "${content:-${first} and ${second}}";
        let mut args = HashMap::new();
        args.insert("first", "one");
        args.insert("second", "two");

        let s = Template::new(test).render(&args);

        assert_eq!(s, "one and two");
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
    fn alt_delimeters2() {
        let mut args = HashMap::new();
        args.insert("dog", "{ good } woofers { eh }");
        args.insert("cat", "${{cat_name:=moggy}} that says ${{cat_noise}}");
        args.insert("cat_noise", "meeow");

        let s = Template::new_delimit("My dog ${{dog}} has a friend {well says he does} ${{cat}}", "${{", "}}").render(&args);

        assert_eq!(s, "My dog { good } woofers { eh } has a friend {well says he does} moggy that says meeow");
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

    #[test]
    fn many() {
        let mut args = HashMap::new();
        args.insert("dog", "woofers|rex");
        args.insert("cat", "kitty|moggi|tiger");
        args.insert("pets", "${dog} and ${cat}");

        let s = Template::new("${*pets}").render(&args);

        assert_eq!(s, "woofers and kitty\nrex and moggi");
    }

    #[test]
    fn many_delim() {
        let mut args = HashMap::new();
        args.insert("dog", "woofers|rex");
        args.insert("cat", "kitty|moggi");
        args.insert("pets", "${dog} and ${cat}");

        let s = Template::new("${*|pets}").render(&args);

        assert_eq!(s, "woofers and kitty|rex and moggi");
    }

    #[test]
    fn many_delim2() {
        let mut args = HashMap::new();
        args.insert("dog", "woofers|rex");
        args.insert("cat", "kitty|moggi");
        args.insert("rabbit", "cuddly");
        args.insert("pets", "${dog}, ${cat} and ${rabbit}");

        let s = Template::new("I love ${*;pets} a lot").render(&args);

        assert_eq!(s, "I love woofers, kitty and cuddly;rex, moggi and cuddly a lot");
    }

    #[test]
    fn many_delim3() {
        let mut args = HashMap::new();
        args.insert("marg", "arg0");
        args.insert("top", "${marg}");

        let s = Template::new("[${*,top}]").render(&args);

        assert_eq!(s, "[arg0]");
    }

    #[test]
    fn many_delim4() {
        let mut args = HashMap::new();
        args.insert("marg", "arg0");
        args.insert("mand_args", r#""${marg}""#);
        args.insert("func", r#"[${*,mand_args}]"#);

        let s = Template::new("${func}").render(&args);
        //let s = Template::new(r#"[${*,mand_args}]"#).render(&args);

        assert_eq!(s, "[\"arg0\"]");
    }

    #[test]
    fn exists() {
        let mut args = HashMap::new();
        args.insert("v1", "aaa");
        args.insert("v2", "bbb");
        args.insert("value", "text");
        let s = Template::new("${?value=text:-${v1}${v2}}").render(&args);

        assert_eq!(s, "aaabbb");
    }

    #[test]
    fn code() {
        let mut args = HashMap::new();

        // Code generate by llmclient
        let code = r####"
Sure, here's a simple example of an HTTP proxy in Rust using the `hyper` and `tokio` crates:
    
    # Cargo.toml
    [package]
    name = "http_proxy"
    version = "0.1.0"
    edition = "2018"
    
    [dependencies]
    hyper = "0.14"
    tokio = { version = "1", features = ["full"] }
    
    // src/main.rs
    use hyper::{Client, Request, Response, Body, Server};
    use hyper::service::{make_service_fn, service_fn};
    use std::convert::Infallible;
    use tokio::runtime::Runtime;
    
    async fn proxy(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
        let client = Client::new();
        client.request(req).await
    }
    
    #[tokio::main]
    async fn main() {
        let make_svc = make_service_fn(|_conn| {
            async {
                Ok::<_, Infallible>(service_fn(proxy))
            }
        });
    
        let addr = ([127, 0, 0, 1], 3000).into();
        let server = Server::bind(&addr).serve(make_svc);
    
        println!("Listening on http://{}", addr);
    
        if let Err(e) = server.await {
            eprintln!("server error: {}", e);
        }
    }
    
    To run this code, ensure you have Rust and Cargo installed, then create a new project, add the dependencies in `Cargo.toml`, and replace the contents of `src/main.rs` with the provided code.
        "####;

        args.insert("code", code);
        let s = Template::new(">>> ${code} <<<").render(&args);

        assert_eq!(s, format!(">>> {} <<<", code.trim()));
    }

    #[test]
    fn code_literal() {
        let mut args = HashMap::new();

        // Code generate by llmclient
        let code = r####"
Sure, here's a simple example of an HTTP proxy in Rust using the `hyper` and `tokio` crates:
    
    # Cargo.toml
    [package]
    name = "http_proxy"
    version = "0.1.0"
    edition = "2018"
    
    [dependencies]
    hyper = "0.14"
    tokio = { version = "1", features = ["full"] }
    ${fred:=FRED}
    
    // src/main.rs
    use hyper::{Client, Request, Response, Body, Server};
    use hyper::service::{make_service_fn, service_fn};
    use std::convert::Infallible;
    use tokio::runtime::Runtime;
    
    async fn proxy(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
        let client = Client::new();
        client.request(req).await
    }
    
    #[tokio::main]
    async fn main() {
        let make_svc = make_service_fn(|_conn| {
            async {
                Ok::<_, Infallible>(service_fn(proxy))
            }
        });
    
        let addr = ([127, 0, 0, 1], 3000).into();
        let server = Server::bind(&addr).serve(make_svc);
    
        println!("Listening on http://{}", addr);
    
        if let Err(e) = server.await {
            eprintln!("server error: {}", e);
        }
    }
    
    To run this code, ensure you have Rust and Cargo installed, then create a new project, add the dependencies in `Cargo.toml`, and replace the contents of `src/main.rs` with the provided code.
        "####;

        args.insert("code", code);
        args.insert("something", "SOMETHING");
        let s = Template::new(">>> ${=code} ${something} <<<").render(&args);

        assert_eq!(s, format!(">>> {} SOMETHING <<<", code));
    }
}
