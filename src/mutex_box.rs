use std::sync::Mutex;

pub struct MutexBox<T> {
    pub mutex: Mutex<Option<T>>,
}

impl<T> MutexBox<T> {
    #[allow(dead_code)]
    pub const fn new() -> Self {
        let mutex: Mutex<Option<T>> = Mutex::new(None);
        MutexBox {
            mutex: mutex,
        }
    }
    #[allow(dead_code)]
    pub const fn new_inited(data:T)->Self{
        let mutex: Mutex<Option<T>> = Mutex::new(Some(data));
        Self {
            mutex: mutex,
        }
    }
    #[allow(dead_code)]
    pub fn open_locked<FunctionLocked, TypeReturn>(
        &self,
        found: FunctionLocked,
        error_val: TypeReturn,
    ) -> TypeReturn
    where
        FunctionLocked: FnOnce(&mut T) -> TypeReturn,
    {
        if let Ok(mut handler_option) = self.mutex.lock() {
            if let Some(handler) = handler_option.as_mut() {
                found(handler)
            } else {
                error_val
            }
        } else {
            error_val
        }
    }

    #[allow(dead_code)]
    pub fn init(&self,data:T) {
        self.mutex
            .lock()
            .expect(format!("could not lock ").as_str())
            .get_or_insert(data);
    }
}
