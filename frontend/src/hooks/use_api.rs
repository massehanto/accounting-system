use leptos::*;

pub fn use_api<T, F, Fut>(api_call: F) -> (ReadSignal<Option<T>>, ReadSignal<bool>, ReadSignal<Option<String>>, Action<(), ()>) 
where 
    T: Clone + 'static,
    F: Fn() -> Fut + 'static + Copy,
    Fut: std::future::Future<Output = Result<T, String>> + 'static,
{
    let data = create_rw_signal(None::<T>);
    let loading = create_rw_signal(false);
    let error = create_rw_signal(None::<String>);
    
    let fetch_action = create_action(move |_: &()| async move {
        loading.set(true);
        error.set(None);
        
        match api_call().await {
            Ok(result) => {
                data.set(Some(result));
            }
            Err(e) => {
                error.set(Some(e));
            }
        }
        
        loading.set(false);
    });
    
    (data.into(), loading.into(), error.into(), fetch_action)
}