// frontend/src/hooks/use_api.rs
use leptos::*;

pub fn use_api<T, F, Fut>(api_call: F) -> (ReadSignal<Option<T>>, ReadSignal<bool>, ReadSignal<Option<String>>, Action<(), ()>) 
where 
    T: Clone + 'static,
    F: Fn() -> Fut + 'static + Copy,
    Fut: std::future::Future<Output = Result<T, String>> + 'static,
{
    let (data, set_data) = create_signal(None::<T>);
    let (loading, set_loading) = create_signal(false);
    let (error, set_error) = create_signal(None::<String>);
    
    let fetch_action = create_action(move |_: &()| async move {
        set_loading.set(true);
        set_error.set(None);
        
        match api_call().await {
            Ok(result) => {
                set_data.set(Some(result));
            }
            Err(e) => {
                set_error.set(Some(e));
            }
        }
        
        set_loading.set(false);
    });
    
    (data, loading, error, fetch_action)
}