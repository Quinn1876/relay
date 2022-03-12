/**
 * The Requests Module's main export is the
 * RequestParser which is an implimentaiton of
 * a Radix tree used to index string queries/requests
 */

#[cfg(test)]
mod test {
    use super::*;

    impl <T>RequestParserResult<T> {
        fn validate_result(&self, other: RequestParserResult<T>) -> &RequestParserResult<T> {
            if std::mem::discriminant(self) != std::mem::discriminant(&other) {
                panic!("Unexpected Result");
            }
            self
        }
    }

    #[test]
    fn get_set_sibling() {
        let mut node: RequestParserNode::<String> = RequestParserNode::new(b'a');
        node.create_sibling(b'b');
        let node = node;
        if let Some(sibling) = node.get_sibling() {
            assert_eq!(sibling.get_key_char(), b'b');
        } else {
            panic!("Unable to get sibling after setting sibling");
        }
    }

    #[test]
    fn get_set_child() {
        let mut node: RequestParserNode::<String> = RequestParserNode::new(b'a');
        node.create_child(b'c');
        let node = node;
        if let Some(child) = node.get_child() {
            assert_eq!(child.get_key_char(), b'c');
        } else {
            panic!("Unable to get child after setting child");
        }
    }
    #[test]
    fn get_insert_one() {
        let mut parser: RequestParser<String> = RequestParser::new();
        parser.insert("HELLO", String::from("World")).validate_result(Success(()));
        let result = match parser.get("HELLO") {
            Success(result) => result,
            _ => panic!("No result")
        };
        assert_eq!(*result, String::from("World"));
    }

    #[test]
    fn insert_single_length_key() {
        let mut parser: RequestParser<String> = RequestParser::new();
        parser.insert("1", String::from("Successful insert")).validate_result(Success(()));
        let result = match parser.get("1") {
            Success(result) => result,
            _ => panic!("no result"),
        };

        assert_eq!(*result, String::from("Successful insert"));
    }

    #[test]
    fn insert_multiple_keys() {
        let mut parser: RequestParser<String> = RequestParser::new();
        parser.insert("1", String::from("Successful insert")).validate_result(Success(()));
        parser.insert("12", String::from("Successful insert")).validate_result(Success(()));
        parser.insert("13", String::from("Successful insert")).validate_result(Success(()));
        parser.insert("14", String::from("Successful insert")).validate_result(Success(()));
        parser.insert("1231", String::from("Successful insert")).validate_result(Success(()));
        parser.insert("3211", String::from("Successful insert")).validate_result(Success(()));
        parser.insert("sdfasdf1", String::from("Successful insert")).validate_result(Success(()));
        parser.insert("gregerdfgdsger1", String::from("Successful insert")).validate_result(Success(()));

        let result = match parser.get("1") {
            Success(result) => result,
            _ => panic!("no result"),
        };

        assert_eq!(*result, String::from("Successful insert"));

        let result = match parser.get("12") {
            Success(result) => result,
            _ => panic!("no result"),
        };

        assert_eq!(*result, String::from("Successful insert"));

        let result = match parser.get("13") {
            Success(result) => result,
            _ => panic!("no result"),
        };

        assert_eq!(*result, String::from("Successful insert"));

        let result = match parser.get("14") {
            Success(result) => result,
            _ => panic!("no result"),
        };

        assert_eq!(*result, String::from("Successful insert"));

        let result = match parser.get("1231") {
            Success(result) => result,
            _ => panic!("no result"),
        };

        assert_eq!(*result, String::from("Successful insert"));

        let result = match parser.get("3211") {
            Success(result) => result,
            _ => panic!("no result"),
        };

        assert_eq!(*result, String::from("Successful insert"));

        let result = match parser.get("sdfasdf1") {
            Success(result) => result,
            _ => panic!("no result"),
        };

        assert_eq!(*result, String::from("Successful insert"));

        let result = match parser.get("gregerdfgdsger1") {
            Success(result) => result,
            _ => panic!("no result"),
        };

        assert_eq!(*result, String::from("Successful insert"));
    }

    #[test]
    fn overwrite_insert() {
        let mut parser: RequestParser::<u32> = RequestParser::new();

        parser.insert("HELLO", 256).validate_result(Success(()));

        let result = match parser.get("HELLO") {
            Success(result) => result,
            _ => panic!("no result"),
        };

        assert_eq!(*result, 256);

        parser.insert("HELLO", 556).validate_result(Success(()));

        let result = match parser.get("HELLO") {
            Success(result) => result,
            _ => panic!("no result"),
        };

        assert_eq!(*result, 556);
    }

    #[test]
    fn strip_line() {
        let mut parser: RequestParser::<u32> = RequestParser::new();
        parser.insert("LINE1\r\n", 7);
        let parser = parser;
        let request = b"LINE1\r\nLINE2\r\n";
        let result = parser.strip_line_and_get_value(&request[..]);

        if let Success((&value, request1)) = result {
            assert_eq!(request1, &b"LINE2\r\n"[..]);
            assert_eq!(value, 7u32);
        } else {
            panic!("Missing Value");
        }
    }
}

pub enum RequestParserResult<T> {
    Success(T),
    EmptyKey,
    InvalidKey,
    InvalidRequest
}
use RequestParserResult::{ Success, EmptyKey, InvalidKey, InvalidRequest };

struct RequestParserNode<T> {
    key_char: u8,
    sibling: Option<Box<RequestParserNode<T>>>,
    child: Option<Box<RequestParserNode<T>>>,
    value: Option<T>
}

pub struct RequestParser<T> {
    children: Vec::<RequestParserNode<T>>,
}

impl<T> RequestParserNode<T> {
    fn create_child(&mut self, key_char: u8) {
        self.child = Some(Box::new(RequestParserNode::new(key_char)));
    }

    fn create_sibling(&mut self, key_char: u8) {
        self.sibling = Some(Box::new(RequestParserNode::new(key_char)));
    }

    fn get_sibling(&self) -> Option<&RequestParserNode<T>> {
        if let Some(sibling) = &self.sibling {
            Some(sibling)
        } else {
            None
        }
    }

    fn get_sibling_mut(&mut self) -> Option<&mut RequestParserNode<T>> {
        if let Some(sibling) = &mut self.sibling {
            Some(sibling)
        } else {
            None
        }
    }

    fn get_sibling_mut_or_create(&mut self, key_char: u8) -> &mut RequestParserNode<T> {
        if self.sibling.is_none() {
            self.create_sibling(key_char);
        }
        self.get_sibling_mut().expect("Sibling should exist")
    }

    fn get_child_mut(&mut self) -> Option<&mut RequestParserNode<T>> {
        if let Some(child) = &mut self.child {
            Some(child)
        } else {
            None
        }
    }

    fn get_child_mut_or_create(&mut self, key_char: u8) -> &mut RequestParserNode<T> {
        if self.child.is_none() {
            self.create_child(key_char);
        }
        return self.get_child_mut().expect("Child Should Exist");
    }

    fn get_child(&self) -> Option<&RequestParserNode<T>> {
        if let Some(child) = &self.child {
            Some(child)
        } else {
            None
        }
    }

    fn get_key_char(&self) -> u8 {
        self.key_char
    }

    fn get_value(&self) -> Option<&T> {
        match &self.value {
            Some(value) => Some(value),
            None => None
        }
    }

    fn new(key_char: u8) -> RequestParserNode<T> {
        RequestParserNode {
            key_char,
            sibling: None,
            child: None,
            value: None
        }
    }

    fn set_value(&mut self, value: T) {
        self.value = Some(value)
    }
}

impl <T> RequestParser<T> {
    pub fn new() -> RequestParser<T> {
        RequestParser {
            children: Vec::<RequestParserNode<T>>::new(),
        }
    }

    fn insert_loop<'a>(&mut self, key: &'a [u8], value: T, root_index: usize) {
        let root = self.children.get_mut(root_index).expect("Invalid array access");
        let mut runner:  &mut RequestParserNode<T> = root;
        for (index, &key_char) in key.iter().enumerate() {
            while runner.get_key_char() != key_char {
                runner = runner.get_sibling_mut_or_create(key_char);
            }

            if index == key.len()-1 {
                runner.set_value(value);
                return ()
            }

            runner = runner.get_child_mut_or_create(
                *key.get(index+1).expect("index should exist")
            );
        }
    }


    /**
     * @brief insert
     * @param key: &str
     * @param value: T
     *
     * Insert value at key. Overwrites previous insertions if there was one
     *
     * @NOTE When constructing the request parser, the most often used commands should be the first ones inserted,
     * this will reduce the time that it takes to parse the most often used commands
     */
    pub fn insert<'a>(&mut self, key: &'a str, value: T) -> RequestParserResult<()> {
        let key = key.as_bytes();
        let key_char = key.first();

        if let Some(&key_char) = key_char {
            let root_index = self.get_root_index(key_char);

            // only happens if a child with the key is not found
            if root_index == self.children.len() {
                self.children.push(
                    RequestParserNode::new(key_char)
                );
            }
            self.insert_loop(key, value, root_index);
        }  else {
            return EmptyKey;
        }

        Success(())
    }

    /**
     * @brief get
     * @param key &'a str
     * get the value associated with the key
     * if the key is blank, then EmptyKey is returned
     * if the key is not valid, aka their is no value at the key, then return InvalidKey
     * else return Success(value)
     */
    pub fn get<'a>(&self, key: &'a str) -> RequestParserResult<&T> {
        /* Initialization */
        if key.len() == 0 {
            return EmptyKey
        }
        let key = key.as_bytes();
        let key_char = key.first().expect("Key is not empty");

        /* Find the index of the root which matches the first byte of the key */
        let root_index = self.get_root_index(*key_char);
        if root_index == key.len() {
            return InvalidKey
        }


        let mut runner = self.children.get(root_index).expect("Index should exist");

        if key.len() == 1 && runner.get_value().is_some() {
            return Success(runner.get_value().expect("Value should be Some"))
        }

        if runner.get_child().is_none() {
            return InvalidKey
        }

        runner = runner.get_child().expect("Child should be Some");

        let (last_key, key_iter) = key.split_last().expect(" Slice should not be empty");

        /* Drill into the tree until the rest of key_iter has been matched */
        if key_iter.len() > 0 {
            for &key_char in &key_iter[1..] {
                while runner.get_key_char() != key_char {
                    if runner.get_sibling().is_none() {
                        return InvalidKey
                    }
                    runner = runner.get_sibling().expect("Sibling is not None");
                }
                if runner.get_child().is_none()  {
                    return InvalidKey
                }
                runner = runner.get_child().expect("Child is not none");
            }
        }
        while runner.get_key_char() != *last_key {
            if runner.get_sibling().is_none() {
                return InvalidKey
            }
            runner = runner.get_sibling().expect("Sibling is not None");
        }
        if runner.get_key_char() == *last_key && runner.get_value().is_some() {
            return Success(runner.get_value().expect("value is some"));
        }
        InvalidKey
    }

    fn get_root_index(&self, key_char: u8) -> usize {
        for (index, child) in self.children.iter().enumerate() {
            if child.get_key_char() == key_char {
                return index;
            }
        }
        return self.children.len();
    }

    /**
     * @brief strip_line_and_get_value
     * @param request: &mut [u8]
     * strips one line from the request and uses that line as the key
     * returns the value at that key
     * If value is None, returns InvalidRequest
     *
     * If the request is Invalid, then request will equal [b'\0']
     */
    pub fn strip_line_and_get_value<'request>(&self, request: &'request [u8]) -> RequestParserResult<(&T, &'request [u8])> {
        let  return_invalid = | request: std::slice::Iter<'_, u8> | -> RequestParserResult<(&T, &[u8])> {
            request.count();
            InvalidRequest
        };

        let mut request = request.iter();

        if let Some(&first) = request.next() {
            let root_index = self.get_root_index(first);
            if root_index != self.children.len() {
                let mut runner = self.children.get(root_index).unwrap();
                while let Some(&key_char) = request.next() {
                    if runner.get_child().is_some() {
                        runner = runner.get_child().unwrap();
                    } else {
                        return return_invalid(request)
                    }
                    while runner.get_key_char() != key_char && runner.get_sibling().is_some(){
                        runner = runner.get_sibling().expect("Sibling should be some");
                    }
                    if runner.get_key_char() != key_char {
                        return return_invalid(request)
                    }
                    if key_char == b'\n' { // End of the line that we are splitting
                        return match runner.get_value() {
                            Some(value) => Success((value, request.as_slice())),
                            None => return_invalid(request)
                        }
                    }
                }
            }
        }

        return_invalid(request)
    }
}

