
#[no_mangle]
pub extern fn add(first: i32, second: i32) -> i32 {
    println!("Inside rust: {} + {}", first, second);
    first + second
}

pub trait Transport {
    fn start(&self);
    fn get_data(&self) -> i64;
}


#[repr(C)]
pub struct MyTransport {
    data: i64
}

impl Transport for MyTransport {
    fn start(&self) {
        println!("MyStransport started with {}", self.data);
    }
    fn get_data(&self) -> i64 {
        self.data
    }
}

#[no_mangle]
pub extern fn create_transport() -> *mut MyTransport {
    println!("Inside create_transport");
    let mut t = Box::new(MyTransport{data: 42});
    return Box::into_raw(t);
}

#[no_mangle]
pub extern fn call_back_from_c(t: *mut MyTransport ){
    unsafe {
        println!("call_back_from_c_with_rust_ptr: {:p}", t);
        println!("call_back_from_c_with_rust_ptr: {}", (*t).get_data());
    }
    //let mut t = Box::new(MyTransport{data: 42});
    //return &mut *t;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
